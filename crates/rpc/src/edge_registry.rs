use crate::error::ClientError;
use edge_registry_client::EdgeRegistryClient;
use tonic::transport::Channel;

tonic::include_proto!("edge_registry");

pub async fn connect(addr: String) -> Result<EdgeRegistryClient<Channel>, ClientError> {
    EdgeRegistryClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "edge registry",
            source: err,
        })
}
