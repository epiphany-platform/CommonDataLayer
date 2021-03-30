use std::collections::HashMap;

use async_graphql::{Context, FieldResult, Object};
use tracing::Instrument;
use utils::message_types::OwnedInsertMessage;
use uuid::Uuid;

use crate::config::Config;
use crate::error::Error;
use crate::schema::context::SchemaRegistryPool;
use crate::schema::utils::{connect_to_cdl_input, get_schema, get_view};
use crate::types::data::InputMessage;
use crate::types::schema::{Definition, FullSchema, NewSchema, NewVersion, UpdateSchema};
use crate::types::view::{NewView, View, ViewUpdate};
use utils::current_timestamp;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn add_schema(&self, context: &Context<'_>, new: NewSchema) -> FieldResult<FullSchema> {
        let span = tracing::trace_span!("add_schema", ?new);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let schema_type: rpc::schema_registry::types::SchemaType = new.schema_type.into();
            let new_id = conn
                .add_schema(rpc::schema_registry::NewSchema {
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: new.name,
                        schema_type: schema_type as i32,
                        insert_destination: new.insert_destination,
                        query_address: new.query_address,
                    },
                    definition: serde_json::to_vec(&new.definition)?,
                })
                .await?
                .into_inner()
                .id;

            let schema = conn
                .get_full_schema(rpc::schema_registry::Id { id: new_id })
                .await?
                .into_inner();

            FullSchema::from_rpc(schema)
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
                definition: rpc::schema_registry::SchemaDefinition {
                    version: new_version.version.clone(),
                    definition: serde_json::to_vec(&new_version.definition)?,
                },
            })
            .await?;

            Ok(Definition {
                definition: new_version.definition,
                version: new_version.version,
            })
        }
        .instrument(span)
        .await
    }

    async fn update_schema(
        &self,
        context: &Context<'_>,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<FullSchema> {
        let span = tracing::trace_span!("update_schema", ?id, ?update);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            conn.update_schema(rpc::schema_registry::SchemaMetadataUpdate {
                id: id.to_string(),
                patch: rpc::schema_registry::SchemaMetadataPatch {
                    name: update.name,
                    insert_destination: update.insert_destination,
                    query_address: update.query_address,
                    schema_type: update.schema_type.map(Into::into),
                },
            })
            .await
            .map_err(rpc::error::registry_error)?;
            get_schema(&mut conn, id).await
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
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let id = conn
                .add_view_to_schema(rpc::schema_registry::NewView {
                    schema_id: schema_id.to_string(),
                    name: new_view.name.clone(),
                    materializer_address: new_view.materializer_address.clone(),
                    fields: new_view.fields.0.clone(),
                })
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner()
                .id;

            Ok(View {
                id: id.parse()?,
                name: new_view.name,
                materializer_address: new_view.materializer_address,
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
        update: ViewUpdate,
    ) -> FieldResult<View> {
        let span = tracing::trace_span!("update_view", ?id, ?update);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let (update_fields, fields) = if let Some(fields) = update.fields {
                (true, fields.0)
            } else {
                (false, HashMap::default())
            };
            conn.update_view(rpc::schema_registry::ViewUpdate {
                id: id.to_string(),
                name: update.name,
                materializer_address: update.materializer_address,
                update_fields,
                fields,
            })
            .await
            .map_err(rpc::error::registry_error)?;

            get_view(&mut conn, id).await
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
