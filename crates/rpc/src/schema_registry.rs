pub mod types;

pub use crate::codegen::schema_registry::*;

use crate::error::ClientError;
use schema_registry_client::SchemaRegistryClient;
use tonic::transport::Channel;

pub async fn connect(addr: String) -> Result<SchemaRegistryClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "schema registry",
            source: err,
        })
}

async fn connect_inner(
    addr: String,
) -> Result<SchemaRegistryClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(SchemaRegistryClient::with_interceptor(
        conn,
        tracing_tools::grpc::interceptor(),
    ))
}
