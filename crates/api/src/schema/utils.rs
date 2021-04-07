use anyhow::Context;
use async_graphql::FieldResult;
use num_traits::FromPrimitive;
use utils::communication::publisher::CommonPublisher;
use uuid::Uuid;

use super::context::SchemaRegistryConn;
use crate::types::schema::*;
use crate::{
    config::{CommunicationMethodConfig, Config},
    error::Error,
};

pub async fn get_view(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<View> {
    tracing::debug!("get view: {:?}", id);
    let view = conn
        .get_view(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    Ok(View {
        id,
        name: view.name,
        materializer_addr: view.materializer_addr,
        fields: serde_json::from_str(&view.fields)?,
    })
}

pub async fn get_schema(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<Schema> {
    tracing::debug!("get schema: {:?}", id);
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    let schema = Schema {
        id,
        name: schema.name,
        insert_destination: schema.insert_destination,
        query_address: schema.query_address,
        schema_type: SchemaType::from_i32(schema.schema_type)
            .ok_or(Error::InvalidSchemaType(schema.schema_type))?,
    };

    tracing::debug!("schema: {:?}", schema);

    Ok(schema)
}

pub async fn connect_to_cdl_input(config: &Config) -> anyhow::Result<CommonPublisher> {
    match config.communication_method.config()? {
        CommunicationMethodConfig::Amqp {
            connection_string, ..
        } => CommonPublisher::new_amqp(&connection_string)
            .await
            .context("Unable to open RabbitMQ publisher for Ingestion Sink"),
        CommunicationMethodConfig::Kafka { brokers, .. } => CommonPublisher::new_kafka(&brokers)
            .await
            .context("Unable to open Kafka publisher for Ingestion Sink"),
        CommunicationMethodConfig::Grpc => CommonPublisher::new_grpc("ingestion-sink")
            .await
            .context("Unable to create GRPC publisher for Ingestion Sink"),
    }
}
