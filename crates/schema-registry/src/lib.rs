#![feature(drain_filter)]

use ::rpc::schema_registry::{Empty, ValueToValidate};
use error::RegistryClientError;
use serde_json::Value;
use tonic::Request;
use types::storage::vertices::View;
use uuid::Uuid;

pub mod db;
pub mod error;
pub mod replication;
pub mod rpc;
pub mod schema;
pub mod types;

pub enum CommunicationMethodConfig {
    Kafka(KafkaConfig),
    Amqp(AmqpConfig),
    Grpc,
}

#[derive(Clone, Debug)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
}

#[derive(Clone, Debug)]
pub struct AmqpConfig {
    pub connection_string: String,
    pub consumer_tag: String,
}

pub async fn validate_data_with_schema(
    schema_id: Uuid,
    json: &Value,
    schema_registry_addr: String,
) -> Result<(), RegistryClientError> {
    let mut client = ::rpc::schema_registry::connect(schema_registry_addr).await?;
    let request = Request::new(ValueToValidate {
        schema_id: schema_id.to_string(),
        value: serde_json::to_string(json).map_err(RegistryClientError::JsonError)?,
    });

    let errors = client.validate_value(request).await?.into_inner().errors;

    if errors.is_empty() {
        Ok(())
    } else {
        Err(RegistryClientError::InvalidData(errors))
    }
}

pub async fn promote_to_master(storage_addr: String) -> Result<String, RegistryClientError> {
    let mut client = ::rpc::schema_registry::connect(storage_addr).await?;
    let response = client.promote_to_master(Request::new(Empty {})).await?;

    Ok(response.into_inner().name)
}

pub async fn heartbeat(storage_addr: String) -> Result<(), RegistryClientError> {
    let mut client = ::rpc::schema_registry::connect(storage_addr).await?;
    client.heartbeat(Request::new(Empty {})).await?;

    Ok(())
}
