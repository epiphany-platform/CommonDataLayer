use anyhow::Context;
use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use cdl_dto::materialization;
use communication_utils::{consumer::ConsumerHandler, message::CommunicationMessage};
use futures::{Stream, StreamExt, TryStreamExt};
use rpc::common::RowDefinition as RpcRowDefinition;
use rpc::materializer_general::{MaterializedView as RpcMaterializedView, Options};
use rpc::object_builder::{object_builder_server::ObjectBuilder, Empty, View};
use rpc::schema_registry::{schema_registry_client::SchemaRegistryClient, types::SchemaType};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, convert::TryInto, pin::Pin};
use std::{collections::HashSet, sync::Arc};
use tonic::transport::Channel;
use utils::metrics::{self, counter};
use uuid::Uuid;

pub mod settings;

#[derive(Clone)]
pub struct ObjectBuilderImpl {
    pool: Pool<SchemaRegistryConnectionManager>,
    chunk_capacity: usize,
}

#[derive(Clone)]
pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

type ObjectStream =
    Pin<Box<dyn Stream<Item = Result<(Uuid, Value), anyhow::Error>> + Send + Sync + 'static>>;
type RowStream =
    Pin<Box<dyn Stream<Item = Result<RowDefinition, anyhow::Error>> + Send + Sync + 'static>>;
type MaterializedChunksStream =
    Pin<Box<dyn Stream<Item = Result<MaterializedView, anyhow::Error>> + Send + Sync + 'static>>;
type MaterializeStream =
    Pin<Box<dyn Stream<Item = Result<RpcRowDefinition, tonic::Status>> + Send + Sync + 'static>>;

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::schema_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.ping(rpc::schema_registry::Empty {})
            .await
            .map_err(|source| rpc::error::ClientError::QueryError { source })?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct MaterializedView {
    view_id: Uuid,
    options: Value,
    rows: Vec<RowDefinition>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct RowDefinition {
    object_id: Uuid,
    fields: HashMap<String, Value>,
}

impl ObjectBuilderImpl {
    pub async fn new(schema_registry_addr: &str, chunk_capacity: usize) -> anyhow::Result<Self> {
        let pool = Pool::builder()
            .build(SchemaRegistryConnectionManager {
                address: schema_registry_addr.to_string(),
            })
            .await
            .unwrap();

        Ok(Self {
            pool,
            chunk_capacity,
        })
    }
}

impl TryInto<RpcRowDefinition> for RowDefinition {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcRowDefinition, Self::Error> {
        let fields = self
            .fields
            .into_iter()
            .map(|(key, value)| Ok((key, serde_json::to_string(&value)?)))
            .collect::<serde_json::Result<_>>()?;
        Ok(RpcRowDefinition {
            object_id: self.object_id.to_string(),
            fields,
        })
    }
}

impl TryInto<RpcMaterializedView> for MaterializedView {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcMaterializedView, Self::Error> {
        let rows = self
            .rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<serde_json::Result<_>>()?;

        Ok(RpcMaterializedView {
            view_id: self.view_id.to_string(),
            options: Options {
                options: serde_json::to_string(&self.options)?,
            },
            rows,
        })
    }
}

#[async_trait]
impl ConsumerHandler for ObjectBuilderImpl {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let payload = msg.payload()?;
        tracing::debug!(?payload, "Handle MQ message");
        counter!("cdl.object-builder.build-object.mq", 1);
        let request: materialization::Request = serde_json::from_str(&payload)?;
        let view_id = request.view_id;

        let view = self.get_view(view_id);
        let chunks = self.build_materialized_chunks(request);

        let (view, mut chunks) = futures::try_join!(view, chunks)?;

        let mut materializer =
            rpc::materializer_general::connect(view.materializer_address).await?;

        while let Some(chunk) = chunks.try_next().await? {
            let rpc_output: RpcMaterializedView = chunk.try_into()?;
            materializer.upsert_view(rpc_output).await?;
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl ObjectBuilder for ObjectBuilderImpl {
    type MaterializeStream = MaterializeStream;

    #[tracing::instrument(skip(self))]
    async fn materialize(
        &self,
        request: tonic::Request<View>,
    ) -> Result<tonic::Response<Self::MaterializeStream>, tonic::Status> {
        let view: View = request.into_inner();

        let request: materialization::Request = view
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("view"))?;

        let rows = self
            .build_rows(request)
            .await
            .map_err(|err| tonic::Status::internal(format!("{}", err)))?;

        let stream = rows
            .map_err(|err| {
                tracing::error!("Could not build materialized row: {:?}", err);
                tonic::Status::internal("Could not build materialized row")
            })
            .and_then(|row| async move {
                row.try_into().map_err(|err| {
                    tracing::error!("Could not serialize materialized row: {:?}", err);
                    tonic::Status::internal("Could not serialize materialized row")
                })
            });

        let stream = Box::pin(stream);

        Ok(tonic::Response::new(stream))
    }

    #[tracing::instrument(skip(self))]
    async fn heartbeat(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        //empty
        Ok(tonic::Response::new(Empty {}))
    }
}

impl ObjectBuilderImpl {
    #[tracing::instrument(skip(self))]
    async fn build_materialized_chunks(
        &self,
        request: materialization::Request,
    ) -> anyhow::Result<MaterializedChunksStream> {
        tracing::debug!(?request, "Handling");

        let view_id = request.view_id;

        // TODO: We download view twice.
        // First time here, second time in `build_rows()`
        // Either use some kind of cache or extract `self.get_view() higher`
        let view = self.get_view(view_id).await?;
        tracing::debug!(?view, "View");

        let rows = self.build_rows(request).await?;
        let rows = rows.chunks(self.chunk_capacity);

        let options: Value = serde_json::from_str(&view.materializer_options)?;

        let chunks = rows.map(move |rows: Vec<anyhow::Result<RowDefinition>>| {
            let rows: Vec<RowDefinition> = rows.into_iter().collect::<anyhow::Result<_>>()?;

            let materialized = MaterializedView {
                view_id,
                options: options.clone(),
                rows,
            };

            Ok(materialized)
        });

        let stream = Box::pin(chunks) as MaterializedChunksStream;

        Ok(stream)
    }

    #[tracing::instrument(skip(self))]
    async fn build_rows(&self, request: materialization::Request) -> anyhow::Result<RowStream> {
        let materialization::Request { view_id, schemas } = request;

        let view = self.get_view(view_id).await?;
        tracing::debug!(?view, "View");

        // TODO: Handle more than one schema

        let objects = self.get_objects(view_id, schemas).await?;

        let fields_defs: HashMap<String, materialization::FieldDefinition> = view
            .fields
            .into_iter()
            .map(|(key, field)| Ok((key, serde_json::from_str(&field)?)))
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        let fields_defs = Arc::new(fields_defs);

        let rows = objects.and_then(move |(object_id, object)| {
            let fields_defs = fields_defs.clone();
            async move { Self::build_row_def(object_id, object, &fields_defs) }
        });

        let rows = Box::pin(rows) as RowStream;

        Ok(rows)
    }

    #[tracing::instrument]
    fn build_row_def(
        object_id: Uuid,
        object: Value,
        fields_defs: &HashMap<String, materialization::FieldDefinition>,
    ) -> anyhow::Result<RowDefinition> {
        use materialization::FieldDefinition::*;

        let object = object
            .as_object()
            .with_context(|| format!("Expected object ({}) to be a JSON object", object_id))?;

        let fields = fields_defs
            .iter()
            .map(|(field_def_key, field_def)| {
                Ok((
                    field_def_key.into(),
                    match field_def {
                        Simple { field_name, .. } => {
                            //TODO: Use field_type
                            let value = object.get(field_name).with_context(|| {
                                format!(
                                    "Object ({}) does not have a field named `{}`",
                                    object_id, field_name
                                )
                            })?;
                            value.clone()
                        }
                        Computed { .. } => {
                            todo!()
                        }
                        Array { .. } => {
                            todo!()
                        }
                    },
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(RowDefinition { object_id, fields })
    }

    #[tracing::instrument(skip(self))]
    async fn get_objects(
        &self,
        view_id: Uuid,
        mut schemas: HashMap<Uuid, materialization::Schema>,
    ) -> anyhow::Result<ObjectStream> {
        if schemas.is_empty() {
            let base_schema = self.get_base_schema_for_view(view_id).await?;
            let base_schema_id: Uuid = base_schema.id.parse()?;
            schemas.insert(base_schema_id, Default::default());
        }

        if schemas.len() == 1 {
            let (schema_id, schema) = schemas.into_iter().next().unwrap();

            self.get_objects_for_ids(schema_id, &schema.object_ids)
                .await
        } else {
            // TODO: Merging more than one schema. Phase II
            // It cannot be empty, because at least one schema has to be assigned to view.
            todo!();
        }
    }

    async fn get_base_schema_for_view(
        &self,
        view_id: Uuid,
    ) -> anyhow::Result<rpc::schema_registry::Schema> {
        let schema = self
            .pool
            .get()
            .await?
            .get_base_schema_of_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schema)
    }

    #[tracing::instrument(skip(self))]
    async fn get_objects_for_ids(
        &self,
        schema_id: Uuid,
        object_ids: &HashSet<Uuid>,
    ) -> anyhow::Result<ObjectStream> {
        let schema_meta = self.get_schema_metadata(schema_id).await?;

        let query_address = schema_meta.query_address.clone();
        let schema_type = schema_meta.schema_type.try_into()?;

        match schema_type {
            SchemaType::DocumentStorage => {
                let values = if object_ids.is_empty() {
                    rpc::query_service::query_by_schema(schema_id.to_string(), query_address).await
                } else {
                    rpc::query_service::query_multiple(
                        object_ids.iter().map(|id| id.to_string()).collect(),
                        query_address,
                    )
                    .await
                }?;

                let stream = Box::pin(values.map_err(anyhow::Error::from).and_then(
                    |object| async move {
                        let id: Uuid = object.object_id.parse()?;
                        let payload: Value = serde_json::from_slice(&object.payload)?;
                        Ok((id, payload))
                    },
                )) as ObjectStream;

                Ok(stream)
            }

            SchemaType::Timeseries => {
                // TODO:
                anyhow::bail!("Timeseries storage is not supported yet")
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_view(&self, view_id: Uuid) -> anyhow::Result<rpc::schema_registry::FullView> {
        let view = self
            .pool
            .get()
            .await?
            .get_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();

        Ok(view)
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_metadata(
        &self,
        schema_id: Uuid,
    ) -> anyhow::Result<rpc::schema_registry::SchemaMetadata> {
        let schema = self
            .pool
            .get()
            .await?
            .get_schema_metadata(rpc::schema_registry::Id {
                id: schema_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schema)
    }
}
