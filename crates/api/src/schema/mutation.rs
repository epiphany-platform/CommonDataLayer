use std::collections::HashMap;

use async_graphql::{Context, FieldResult, Object};
use tracing::Instrument;
use utils::message_types::OwnedInsertMessage;
use uuid::Uuid;

use crate::schema::context::{EdgeRegistryPool, SchemaRegistryPool};
use crate::schema::utils::{connect_to_cdl_input, get_schema, get_view};
use crate::types::data::{InputMessage, ObjectRelations};
use crate::types::schema::{Definition, FullSchema, NewSchema, NewVersion, UpdateSchema};
use crate::types::view::{NewView, View, ViewUpdate};
use crate::{config::Config, error::Error};
use utils::current_timestamp;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn add_schema(&self, context: &Context<'_>, new: NewSchema) -> FieldResult<FullSchema> {
        let span = tracing::trace_span!("add_schema", ?new);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let new_id = conn
                .add_schema(rpc::schema_registry::NewSchema {
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: new.name,
                        schema_type: new.schema_type.into(),
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
                    materializer_options: serde_json::to_string(&new_view.materializer_options)?,
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
                materializer_options: new_view.materializer_options,
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
                name: update.name.clone(),
                materializer_address: update.materializer_address.clone(),
                materializer_options: update
                    .materializer_options
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?
                    .unwrap_or_default(),
                fields,
                update_fields,
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

    /// Add new relation, return generated `relation_id`
    async fn add_relation(
        &self,
        context: &Context<'_>,
        parent_schema_id: Uuid,
        child_schema_id: Uuid,
    ) -> FieldResult<Uuid> {
        let span = tracing::trace_span!("add_relation", ?parent_schema_id, ?child_schema_id);
        async move {
            let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
            Ok(conn
                .add_relation(rpc::edge_registry::SchemaRelation {
                    parent_schema_id: parent_schema_id.to_string(),
                    child_schema_id: child_schema_id.to_string(),
                })
                .await?
                .into_inner()
                .relation_id
                .parse()?)
        }
        .instrument(span)
        .await
    }

    /// Add new object-object edges
    async fn add_edges(
        &self,
        context: &Context<'_>,
        relations: Vec<ObjectRelations>,
    ) -> FieldResult<bool> {
        let span = tracing::trace_span!("add_edges", len = relations.len());
        async move {
            let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
            conn.add_edges(rpc::edge_registry::ObjectRelations {
                relations: relations
                    .into_iter()
                    .map(|relation| rpc::edge_registry::Edge {
                        relation_id: relation.relation_id.to_string(),
                        parent_object_id: relation.parent_object_id.to_string(),
                        child_object_ids: relation
                            .child_object_ids
                            .into_iter()
                            .map(|id| id.to_string())
                            .collect(),
                    })
                    .collect(),
            })
            .await?;

            Ok(true)
        }
        .instrument(span)
        .await
    }
}
