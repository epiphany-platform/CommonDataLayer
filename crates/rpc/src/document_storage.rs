use crate::error::ClientError;
use document_storage_client::DocumentStorageClient;
use tonic::transport::Channel;

tonic::include_proto!("document_storage");

pub async fn connect(addr: String) -> Result<DocumentStorageClient<Channel>, ClientError> {
    DocumentStorageClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "document storage",
            source: err,
        })
}
