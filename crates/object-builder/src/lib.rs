pub mod args;

use std::collections::HashMap;

use anyhow::Context;
use async_trait::async_trait;
use rpc::object_builder::object_builder_server::ObjectBuilder;
use rpc::object_builder::{MaterializedView, RowDefinition as RpcRowDefinition, ViewId};
use rpc::schema_registry::ViewSchema;
use rpc::schema_registry::{schema_registry_client::SchemaRegistryClient, types::SchemaType};
use serde::Serialize;
use serde_json::Value;
use tonic::transport::Channel;
use utils::metrics::{self, counter};
use utils::{
    communication::{consumer::ConsumerHandler, message::CommunicationMessage},
    types::FieldDefinition,
};
use uuid::Uuid;

use crate::args::Args;

#[derive(Clone)]
pub struct ObjectBuilderImpl {
    schema_registry: SchemaRegistryClient<Channel>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Output {
    view_id: Uuid,
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
        let schema_registry =
            rpc::schema_registry::connect(args.schema_registry_addr.clone()).await?;

        Ok(Self { schema_registry })
    }
}

#[tonic::async_trait]
impl ObjectBuilder for ObjectBuilderImpl {
    async fn materialize(
        &self,
        request: tonic::Request<ViewId>,
    ) -> Result<tonic::Response<MaterializedView>, tonic::Status> {
        let view_id: Uuid = request
            .into_inner()
            .view_id
            .parse()
            .map_err(|_| tonic::Status::invalid_argument("view_id"))?;

        let object = self
            .build_object(view_id)
            .await
            .map_err(|err| tonic::Status::internal(format!("{}", err)))?;

        let rows = object
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
            .collect::<serde_json::Result<_>>()
            .map_err(|err| {
                tracing::error!("Could not serialize view fields: {:?}", err);
                tonic::Status::internal("Could not serialize view fields")
            })?;

        Ok(tonic::Response::new(MaterializedView {
            view_id: object.view_id.to_string(),
            rows,
        }))
    }
}

#[async_trait]
impl ConsumerHandler for ObjectBuilderImpl {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        counter!("cdl.object-builder.build-object.mq", 1);
        let view_id: Uuid = msg.payload()?.parse()?;

        let view = self.get_view(&view_id);
        let output = self.build_object(view_id);

        let (view, _output) = futures::try_join!(view, output)?;

        let _materializer_addr = view.materializer_addr;
        // TODO: Sending output to materializer
        // via GRPC

        Ok(())
    }
}

impl ObjectBuilderImpl {
    async fn build_object(&self, view_id: Uuid) -> anyhow::Result<Output> {
        tracing::debug!(?view_id, "Handling");

        let view = self.get_view(&view_id);
        let base_schema = self.get_base_schema(&view_id);
        let (view, base_schema) = futures::try_join!(view, base_schema)?;

        tracing::debug!(?view, ?base_schema, "View");

        let fields_defs: HashMap<String, FieldDefinition> = serde_json::from_str(&view.fields)?;
        let objects = self.get_objects(&base_schema).await?;
        tracing::debug!(?objects, "Objects");

        let rows = objects
            .into_iter()
            .map(|(object_id, object)| Self::build_row_def(object_id, object, &fields_defs))
            .collect::<anyhow::Result<_>>()?;

        let output = Output { view_id, rows };

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
    async fn get_objects(&self, base_schema: &ViewSchema) -> anyhow::Result<HashMap<Uuid, Value>> {
        let schema_id = &base_schema.schema_id;
        let query_address = &base_schema.schema.query_address;
        let schema_type = base_schema.schema.schema_type().into();

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
            .schema_registry
            .clone()
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
    ) -> anyhow::Result<rpc::schema_registry::ViewSchema> {
        let schemas = self
            .schema_registry
            .clone()
            .get_base_schema_of_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schemas)
    }
}