use juniper::FieldResult;
use uuid::Uuid;
use anyhow::Context;
use async_graphql::FieldResult;
use num_traits::FromPrimitive;
use utils::communication::publisher::CommonPublisher;

use crate::schema::context::SchemaRegistryConn;
use crate::types::schema::FullSchema;
use crate::types::view::View;
use crate::{
    config::{CommunicationMethodConfig, Config},
};

pub async fn get_view(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<View> {
    tracing::debug!("get view: {:?}", id);
    let view = conn
        .get_view(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    View::from_rpc(view)
}

pub async fn get_schema(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<FullSchema> {
    tracing::debug!("get schema: {:?}", id);
    let schema = conn
        .get_full_schema(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    FullSchema::from_rpc(schema)
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
