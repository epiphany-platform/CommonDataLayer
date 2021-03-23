use juniper::{graphql_object, FieldResult};
use serde_json::value::{RawValue, Value};
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
        log::debug!("add schema {:?}", new);

        let mut conn = context.connect_to_registry().await?;
        let r#type: rpc::schema_registry::types::SchemaType = new.r#type.into();
        let new_id = conn
            .add_schema(rpc::schema_registry::NewSchema {
                metadata: rpc::schema_registry::SchemaMetadata {
                    name: new.name,
                    r#type: r#type as i32,
                    topic_or_queue: new.topic_or_queue,
                    query_address: new.query_address,
                },
                definition: serde_json::to_vec(&serde_json::from_str::<Value>(&new.definition)?)?,
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

    async fn add_schema_definition(
        context: &Context,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        log::debug!(
            "add schema definition for {:?} - {:?}",
            schema_id,
            new_version
        );

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

    async fn update_schema(
        context: &Context,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<SchemaWithDefinitions> {
        log::debug!("update schema for {} - {:?}", id, update);

        let mut conn = context.connect_to_registry().await?;
        let r#type: Option<rpc::schema_registry::types::SchemaType> =
            update.r#type.map(|st| st.into());
        conn.update_schema(rpc::schema_registry::SchemaMetadataUpdate {
            id: id.to_string(),
            patch: rpc::schema_registry::SchemaMetadataPatch {
                name: update.name,
                query_address: update.query_address,
                topic_or_queue: update.topic_or_queue,
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

    async fn insert_message(context: &Context, message: InputMessage) -> FieldResult<bool> {
        log::debug!(
            "inserting single message with ID {} for schema {}",
            message.object_id,
            message.schema_id
        );

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

    async fn insert_batch(context: &Context, messages: Vec<InputMessage>) -> FieldResult<bool> {
        log::debug!("inserting batch of {} messages", messages.len());

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
}
