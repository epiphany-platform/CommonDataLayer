use crate::error::ClientError;
use general_materializer_client::GeneralMaterializerClient;
use tonic::transport::Channel;

pub use crate::codegen::materializer_general::*;

pub async fn connect(addr: String) -> Result<GeneralMaterializerClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<GeneralMaterializerClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(GeneralMaterializerClient::with_interceptor(
        conn,
        tracing_utils::grpc::interceptor(),
    ))
}
