use crate::error::ClientError;
use generic_rpc_client::GenericRpcClient;
use tonic::transport::Channel;

pub use crate::codegen::generic_rpc::*;

pub async fn connect(addr: String) -> Result<GenericRpcClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(addr: String) -> Result<GenericRpcClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(GenericRpcClient::with_interceptor(
        conn,
        tracing_tools::grpc::interceptor(),
    ))
}
