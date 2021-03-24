use juniper::{graphql_object, FieldResult};
use serde_json::value::{RawValue, Value};
use tracing::Instrument;
use utils::message_types::DataRouterInsertMessage;
use uuid::Uuid;

use crate::error::Error;
use crate::schema::context::Context;
use crate::types::data::InputMessage;
use crate::types::schema::{
    Definition, NewSchema, NewVersion, SchemaWithDefinitions, UpdateSchema,
};

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn add_schema(context: &Context, new: NewSchema) -> FieldResult<SchemaWithDefinitions> {
        let span = tracing::trace_span!("add_schema", ?new);
        async move {
            let mut conn = context.connect_to_registry().await?;

            let r#type: rpc::schema_registry::types::SchemaType = new.r#type.into();
            let new_id = conn
                .add_schema(rpc::schema_registry::NewSchema {
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: new.name,
                        r#type: r#type as i32,
                        insert_destination: new.insert_destination,
                        query_address: new.query_address,
                    },
                    definition: serde_json::to_vec(&serde_json::from_str::<Value>(
                        &new.definition,
                    )?)?,
                })
                .await?
                .into_inner()
                .id;

            let schema = conn
                .get_schema_with_definitions(rpc::schema_registry::Id { id: new_id })
                .await?
                .into_inner();

            SchemaWithDefinitions::from_rpc(schema)
        }
        .instrument(span)
        .await
    }

    async fn add_schema_definition(
        context: &Context,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        let span = tracing::trace_span!("add_schema_definition", ?schema_id, ?new_version);
        async move {
            let mut conn = context.connect_to_registry().await?;
            conn.add_schema_version(rpc::schema_registry::NewSchemaVersion {
                id: schema_id.to_string(),
                definition: rpc::schema_registry::SchemaDefinition {
                    version: new_version.version.clone(),
                    definition: serde_json::to_vec(&serde_json::from_str::<Value>(
                        &new_version.definition,
                    )?)?,
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
        context: &Context,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<SchemaWithDefinitions> {
        let span = tracing::trace_span!("update_schema", ?id, ?update);
        async move {
            let mut conn = context.connect_to_registry().await?;

            let r#type: Option<rpc::schema_registry::types::SchemaType> =
                update.r#type.map(|st| st.into());
            conn.update_schema(rpc::schema_registry::SchemaMetadataUpdate {
                id: id.to_string(),
                patch: rpc::schema_registry::SchemaMetadataPatch {
                    name: update.name,
                    query_address: update.query_address,
                    insert_destination: update.insert_destination,
                    r#type: r#type.map(|st| st as i32),
                },
            })
            .await?;

            let schema = conn
                .get_schema_with_definitions(rpc::schema_registry::Id { id: id.to_string() })
                .await?
                .into_inner();

            SchemaWithDefinitions::from_rpc(schema)
        }
        .instrument(span)
        .await
    }

    async fn insert_message(context: &Context, message: InputMessage) -> FieldResult<bool> {
        let span = tracing::trace_span!("insert_message", ?message.object_id, ?message.schema_id);
        async move {
            let publisher = context.connect_to_cdl_input().await?;
            let payload = serde_json::to_vec(&DataRouterInsertMessage {
                object_id: message.object_id,
                schema_id: message.schema_id,
                data: &RawValue::from_string(message.payload)?,
            })?;

            publisher
                .publish_message(&context.config().insert_destination, "", payload)
                .await
                .map_err(Error::PublisherError)?;
            Ok(true)
        }
        .instrument(span)
        .await
    }

    async fn insert_batch(context: &Context, messages: Vec<InputMessage>) -> FieldResult<bool> {
        let span = tracing::trace_span!("insert_batch", len = messages.len());
        async move {
            let publisher = context.connect_to_cdl_input().await?;
            let order_group_id = Uuid::new_v4().to_string();

            for message in messages {
                let payload = serde_json::to_vec(&DataRouterInsertMessage {
                    object_id: message.object_id,
                    schema_id: message.schema_id,
                    data: &RawValue::from_string(message.payload)?,
                })?;

                publisher
                    .publish_message(
                        &context.config().insert_destination,
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
