use crate::error::ClientError;
use blob_storage_client::BlobStorageClient;
use tonic::transport::Channel;

tonic::include_proto!("blob_storage");

pub async fn connect(addr: String) -> Result<BlobStorageClient<Channel>, ClientError> {
    BlobStorageClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "blob storage",
            source: err,
        })
}
