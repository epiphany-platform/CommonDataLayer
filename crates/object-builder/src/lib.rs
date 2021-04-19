pub mod args;

use std::{collections::HashMap, convert::TryInto};

use anyhow::Context;
use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use serde::Serialize;
use serde_json::Value;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::args::Args;
use rpc::common::MaterializedView;
use rpc::common::RowDefinition as RpcRowDefinition;
use rpc::object_builder::{object_builder_server::ObjectBuilder, Empty, ViewId};
use rpc::schema_registry::{
    schema_registry_client::SchemaRegistryClient, types::SchemaType, Schema,
};
use utils::metrics::{self, counter};
use utils::{
    communication::{consumer::ConsumerHandler, message::CommunicationMessage},
    types::FieldDefinition,
};

#[derive(Clone)]
pub struct ObjectBuilderImpl {
    pool: Pool<SchemaRegistryConnectionManager>,
}

#[derive(Clone)]
pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

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
            .map_err(rpc::error::schema_registry_error)?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Output {
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
    pub async fn new(args: &Args) -> anyhow::Result<Self> {
        let pool = Pool::builder()
            .build(SchemaRegistryConnectionManager {
                address: args.schema_registry_addr.clone(),
            })
            .await
            .unwrap();

        Ok(Self { pool })
    }
}

impl TryInto<MaterializedView> for Output {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<MaterializedView, Self::Error> {
        let rows = self
            .rows
            .into_iter()
            .map(|row| {
                let fields = row
                    .fields
                    .into_iter()
                    .map(|(key, value)| Ok((key, serde_json::to_string(&value)?)))
                    .collect::<serde_json::Result<_>>()?;
                Ok(RpcRowDefinition {
                    object_id: row.object_id.to_string(),
                    fields,
                })
            })
            .collect::<serde_json::Result<_>>()?;

        Ok(MaterializedView {
            view_id: self.view_id.to_string(),
            options: serde_json::to_string(&self.options)?,
            rows,
        })
    }
}

#[tonic::async_trait]
impl ObjectBuilder for ObjectBuilderImpl {
    #[tracing::instrument(skip(self))]
    async fn materialize(
        &self,
        request: tonic::Request<ViewId>,
    ) -> Result<tonic::Response<MaterializedView>, tonic::Status> {
        utils::tracing::grpc::set_parent_span(&request);

        let view_id: Uuid = request
            .into_inner()
            .view_id
            .parse()
            .map_err(|_| tonic::Status::invalid_argument("view_id"))?;

        let object = self
            .build_object(view_id)
            .await
            .map_err(|err| tonic::Status::internal(format!("{}", err)))?;

        let rpc_object = object.try_into().map_err(|err| {
            tracing::error!("Could not serialize materialized view: {:?}", err);
            tonic::Status::internal("Could not serialize materialized view")
        })?;

        Ok(tonic::Response::new(rpc_object))
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

#[async_trait]
impl ConsumerHandler for ObjectBuilderImpl {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let payload = msg.payload()?;
        tracing::debug!(?payload, "Handle MQ message");
        counter!("cdl.object-builder.build-object.mq", 1);
        let view_id: Uuid = payload.trim().parse()?;

        let view = self.get_view(&view_id);
        let object = self.build_object(view_id);

        let (view, object) = futures::try_join!(view, object)?;

        let rpc_object: MaterializedView = object.try_into()?;

        rpc::materializer::connect(view.materializer_address)
            .await?
            .upsert_view(utils::tracing::grpc::inject_span(rpc_object))
            .await?;

        Ok(())
    }
}

impl ObjectBuilderImpl {
    #[tracing::instrument(skip(self))]
    async fn build_object(&self, view_id: Uuid) -> anyhow::Result<Output> {
        tracing::debug!(?view_id, "Handling");

        let view = self.get_view(&view_id);
        let base_schema = self.get_base_schema(&view_id);
        let (view, base_schema) = futures::try_join!(view, base_schema)?;

        tracing::debug!(?view, ?base_schema, "View");

        let options = serde_json::from_str(&view.materializer_options)?;

        let fields_defs: HashMap<String, FieldDefinition> = view
            .fields
            .into_iter()
            .map(|(key, value)| Ok((key, serde_json::from_str(&value)?)))
            .collect::<anyhow::Result<_>>()?;

        let objects = self.get_objects(&base_schema).await?;
        tracing::debug!(?objects, "Objects");

        let rows = objects
            .into_iter()
            .map(|(object_id, object)| Self::build_row_def(object_id, object, &fields_defs))
            .collect::<anyhow::Result<_>>()?;

        let output = Output {
            view_id,
            options,
            rows,
        };

        tracing::debug!(?output, "Output");

        Ok(output)
    }

    #[tracing::instrument]
    fn build_row_def(
        object_id: Uuid,
        object: Value,
        fields_defs: &HashMap<String, FieldDefinition>,
    ) -> anyhow::Result<RowDefinition> {
        let object = object
            .as_object()
            .with_context(|| format!("Expected object ({}) to be a JSON object", object_id))?;

        let fields = fields_defs
            .iter()
            .map(|(field_def_key, field_def)| {
                Ok((
                    field_def_key.into(),
                    match field_def {
                        FieldDefinition::FieldName(field_name) => {
                            let value = object.get(field_name).with_context(|| {
                                format!(
                                    "Object ({}) does not have a field named `{}`",
                                    object_id, field_name
                                )
                            })?;
                            value.clone()
                        }
                    },
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(RowDefinition { object_id, fields })
    }

    #[tracing::instrument(skip(self))]
    async fn get_objects(&self, base_schema: &Schema) -> anyhow::Result<HashMap<Uuid, Value>> {
        let schema_id = &base_schema.id;
        let query_address = &base_schema.metadata.query_address;
        let schema_type = base_schema.metadata.schema_type.try_into()?;

        match schema_type {
            SchemaType::DocumentStorage => {
                let values = rpc::query_service::query_by_schema(
                    schema_id.to_string(),
                    query_address.into(),
                )
                .await?;
                values
                    .into_iter()
                    .map(|(object_id, value)| {
                        let id: Uuid = object_id.parse()?;
                        Ok((id, serde_json::from_slice(&value)?))
                    })
                    .collect()
            }
            SchemaType::Timeseries => {
                anyhow::bail!("Timeseries storage is not supported yet")
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_view(&self, view_id: &Uuid) -> anyhow::Result<rpc::schema_registry::View> {
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
    async fn get_base_schema(
        &self,
        view_id: &Uuid,
    ) -> anyhow::Result<rpc::schema_registry::Schema> {
        let schemas = self
            .pool
            .get()
            .await?
            .get_base_schema_of_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schemas)
    }
}
