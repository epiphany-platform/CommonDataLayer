use juniper::{graphql_object, FieldResult};
use num_traits::ToPrimitive;
use serde_json::value::RawValue;
use utils::message_types::DataRouterInsertMessage;
use uuid::Uuid;

use crate::error::Error;
use crate::schema::context::Context;
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::InputMessage;
use crate::types::schema::*;

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn add_schema(context: &Context, new: NewSchema) -> FieldResult<Schema> {
        log::debug!("add schema {:?}", new);
        let mut conn = context.connect_to_registry().await?;

        let NewSchema {
            name,
            query_address,
            topic,
            definition,
            schema_type,
        } = new;

        let rpc_schema_type: i32 = schema_type.to_i32().unwrap(); // Unwrap because we for sure can build i32 from enum

        let id = conn
            .add_schema(rpc::schema_registry::NewSchema {
                id: "".into(),
                name: name.clone(),
                query_address: query_address.clone(),
                topic: topic.clone(),
                definition,
                schema_type: rpc_schema_type,
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .id
            .parse()?;

        Ok(Schema {
            id,
            name,
            query_address,
            topic,
            schema_type,
        })
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

        let NewVersion {
            definition,
            version,
        } = new_version;

        conn.add_schema_version(rpc::schema_registry::NewSchemaVersion {
            id: schema_id.to_string(),
            version: version.clone(),
            definition: definition.clone(),
        })
        .await
        .map_err(rpc::error::registry_error)?;

        Ok(Definition {
            definition,
            version,
        })
    }

    async fn add_view(context: &Context, schema_id: Uuid, new_view: NewView) -> FieldResult<View> {
        log::debug!("add view for {} - {:?}", schema_id, new_view);

        let NewView { name, expression } = new_view.clone();
        let mut conn = context.connect_to_registry().await?;
        let id = conn
            .add_view_to_schema(rpc::schema_registry::NewSchemaView {
                schema_id: schema_id.to_string(),
                view_id: "".into(),
                name,
                jmespath: expression,
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .id;

        Ok(View {
            id: id.parse()?,
            name: new_view.name,
            expression: new_view.expression,
        })
    }

    async fn update_view(context: &Context, id: Uuid, update: UpdateView) -> FieldResult<View> {
        log::debug!("update view for {} - {:?}", id, update);

        let mut conn = context.connect_to_registry().await?;

        let UpdateView { name, expression } = update;

        conn.update_view(rpc::schema_registry::UpdatedView {
            id: id.to_string(),
            name: name.clone(),
            jmespath: expression.clone(),
        })
        .await
        .map_err(rpc::error::registry_error)?;

        get_view(&mut conn, id).await
    }

    async fn update_schema(
        context: &Context,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<Schema> {
        log::debug!("update schema for {} - {:?}", id, update);

        let mut conn = context.connect_to_registry().await?;

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
