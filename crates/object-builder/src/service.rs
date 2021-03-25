use std::collections::HashMap;

use anyhow::Context;
use async_trait::async_trait;
use rpc::schema_registry::ViewSchema;
use rpc::schema_registry::{schema_registry_client::SchemaRegistryClient, types::SchemaType};
use serde::Serialize;
use serde_json::Value;
use tonic::transport::Channel;
use utils::{
    communication::{
        message::CommunicationMessage,
        parallel_consumer::{
            ParallelCommonConsumer, ParallelCommonConsumerConfig, ParallelConsumerHandler,
        },
    },
    types::FieldDefinition,
};
use uuid::Uuid;

pub struct Service {
    consumer: ParallelCommonConsumer,
    handler: ServiceHandler,
}

struct ServiceHandler {
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

#[async_trait]
impl ParallelConsumerHandler for ServiceHandler {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let view_id: Uuid = msg.payload()?.parse()?;

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

        let _materializer_addr = view.materializer_addr;
        // TODO: Sending output to materializer
        // Via GRPC?

        tracing::debug!(?output, "Output");

        Ok(())
    }
}

impl ServiceHandler {
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
            .clone() // Client is using Arc<> internally therefore `.clone()`
            // is pretty cheap and recommended way to reuse connection between threads
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

impl Service {
    pub async fn new(
        config: ParallelCommonConsumerConfig<'_>,
        schema_registry_addr: &str,
    ) -> anyhow::Result<Self> {
        let consumer = ParallelCommonConsumer::new(config).await?;

        let schema_registry = rpc::schema_registry::connect(schema_registry_addr.into()).await?;

        Ok(Self {
            consumer,
            handler: ServiceHandler { schema_registry },
        })
    }

    pub async fn listen(self) -> anyhow::Result<()> {
        self.consumer.par_run(self.handler).await?;

        tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

        Ok(())
    }
}
