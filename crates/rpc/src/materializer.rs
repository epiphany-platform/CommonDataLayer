use crate::error::ClientError;
use materializer_client::MaterializerClient;
use tonic::transport::Channel;

tonic::include_proto!("materializer");

pub async fn connect(addr: String) -> Result<MaterializerClient<Channel>, ClientError> {
    MaterializerClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "materializer",
            source: err,
        })
}
