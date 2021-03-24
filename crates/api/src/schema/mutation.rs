use async_graphql::{Context, FieldResult, Object};
use num_traits::ToPrimitive;
use tracing::Instrument;
use utils::message_types::OwnedInsertMessage;
use uuid::Uuid;

use crate::schema::context::SchemaRegistryPool;
use crate::schema::utils::{connect_to_cdl_input, get_schema, get_view};
use crate::types::data::InputMessage;
use crate::types::schema::*;
use crate::{config::Config, error::Error};
use utils::current_timestamp;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn add_schema(&self, context: &Context<'_>, new: NewSchema) -> FieldResult<Schema> {
        let span = tracing::trace_span!("add_schema", ?new);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let rpc_schema_type: i32 = new.schema_type.to_i32().unwrap(); // Unwrap because we for sure can build i32 from enum

            let id = conn
                .add_schema(rpc::schema_registry::NewSchema {
                    id: "".into(),
                    name: new.name.clone(),
                    query_address: new.query_address.clone(),
                    topic: new.topic.clone(),
                    definition: serde_json::to_string(&new.definition)?,
                    schema_type: rpc_schema_type,
                })
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner()
                .id
                .parse()?;

            Ok(Schema {
                id,
                name: new.name,
                topic: new.topic,
                query_address: new.query_address,
                schema_type: new.schema_type,
            })
        }
        .instrument(span)
        .await
    }

    async fn add_schema_definition(
        &self,
        context: &Context<'_>,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        let span = tracing::trace_span!("add_schema_definition", ?schema_id, ?new_version);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            conn.add_schema_version(rpc::schema_registry::NewSchemaVersion {
                id: schema_id.to_string(),
                version: new_version.version.clone(),
                definition: serde_json::to_string(&new_version.definition)?,
            })
            .await
            .map_err(rpc::error::registry_error)?;

            Ok(Definition {
                definition: new_version.definition,
                version: new_version.version,
            })
        }
        .instrument(span)
        .await
    }

    async fn add_view(
        &self,
        context: &Context<'_>,
        schema_id: Uuid,
        new_view: NewView,
    ) -> FieldResult<View> {
        let span = tracing::trace_span!("add_view", ?schema_id, ?new_view);
        async move {
            let NewView {
                name,
                materializer_addr,
                fields,
            } = new_view.clone();
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
            let id = conn
                .add_view_to_schema(rpc::schema_registry::NewSchemaView {
                    schema_id: schema_id.to_string(),
                    view_id: "".into(),
                    name,
                    materializer_addr,
                    fields,
                })
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner()
                .id;

            Ok(View {
                id: id.parse()?,
                name: new_view.name,
                materializer_addr: new_view.materializer_addr,
                fields: new_view.fields,
            })
        }
        .instrument(span)
        .await
    }

    async fn update_view(
        &self,
        context: &Context<'_>,
        id: Uuid,
        update: UpdateView,
    ) -> FieldResult<View> {
        let span = tracing::trace_span!("update_view", ?id, ?update);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let UpdateView {
                name,
                materializer_addr,
                fields,
            } = update;

            conn.update_view(rpc::schema_registry::UpdatedView {
                id: id.to_string(),
                name: name.clone(),
                materializer_addr: materializer_addr.clone(),
                fields: fields.clone(),
            })
            .await
            .map_err(rpc::error::registry_error)?;

            get_view(&mut conn, id).await
        }
        .instrument(span)
        .await
    }

    async fn update_schema(
        &self,
        context: &Context<'_>,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<Schema> {
        let span = tracing::trace_span!("update_schema", ?id, ?update);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let UpdateSchema {
                name,
                query_address: address,
                topic,
                schema_type,
            } = update;

            conn.update_schema_metadata(rpc::schema_registry::SchemaMetadataUpdate {
                id: id.to_string(),
                name,
                address,
                topic,
                schema_type: schema_type.and_then(|s| s.to_i32()),
            })
            .await
            .map_err(rpc::error::registry_error)?;
            get_schema(&mut conn, id).await
        }
        .instrument(span)
        .await
    }

    async fn insert_message(
        &self,
        context: &Context<'_>,
        message: InputMessage,
    ) -> FieldResult<bool> {
        let span = tracing::trace_span!("insert_message", ?message.object_id, ?message.schema_id);
        async move {
            let publisher = connect_to_cdl_input(context.data_unchecked::<Config>()).await?;
            let payload = serde_json::to_vec(&OwnedInsertMessage {
                object_id: message.object_id,
                schema_id: message.schema_id,
                data: message.payload.0,
                timestamp: current_timestamp(),
            })?;

            publisher
                .publish_message(
                    &context.data_unchecked::<Config>().insert_destination,
                    "",
                    payload,
                )
                .await
                .map_err(Error::PublisherError)?;
            Ok(true)
        }
        .instrument(span)
        .await
    }

    async fn insert_batch(
        &self,
        context: &Context<'_>,
        messages: Vec<InputMessage>,
    ) -> FieldResult<bool> {
        let span = tracing::trace_span!("insert_batch", len = messages.len());
        async move {
            let publisher = connect_to_cdl_input(context.data_unchecked::<Config>()).await?;
            let order_group_id = Uuid::new_v4().to_string();

            for message in messages {
                let payload = serde_json::to_vec(&OwnedInsertMessage {
                    object_id: message.object_id,
                    schema_id: message.schema_id,
                    data: message.payload.0,
                    timestamp: current_timestamp(),
                })?;

                publisher
                    .publish_message(
                        &context.data_unchecked::<Config>().insert_destination,
                        &order_group_id,
                        payload,
                    )
                    .await
                    .map_err(Error::PublisherError)?;
            }
            Ok(true)
        }
        .instrument(span)
        .await
    }
}
