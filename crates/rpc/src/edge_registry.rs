use crate::error::ClientError;
use edge_registry_client::EdgeRegistryClient;
use tonic::transport::Channel;

pub use crate::codegen::edge_registry::*;

pub async fn connect(addr: String) -> Result<EdgeRegistryClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "edge registry",
            source: err,
        })
}

async fn connect_inner(
    addr: String,
) -> Result<EdgeRegistryClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(EdgeRegistryClient::with_interceptor(
        conn,
        tracing_tools::grpc::interceptor(),
    ))
}
