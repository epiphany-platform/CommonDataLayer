use crate::error::ClientError;
use on_demand_materializer_client::OnDemandMaterializerClient;
use tonic::transport::Channel;

pub use crate::codegen::materializer_ondemand::*;

pub async fn connect(addr: String) -> Result<OnDemandMaterializerClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<OnDemandMaterializerClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(OnDemandMaterializerClient::with_interceptor(
        conn,
        tracing_utils::grpc::interceptor(),
    ))
}
