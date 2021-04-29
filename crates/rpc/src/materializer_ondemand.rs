use crate::error::ClientError;
use on_demand_materializer_client::OnDemandMaterializerClient;
use tonic::transport::Channel;

pub use crate::codegen::materializer_ondemand::*;

pub async fn connect(addr: String) -> Result<OnDemandMaterializerClient<Channel>, ClientError> {
    OnDemandMaterializerClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}
