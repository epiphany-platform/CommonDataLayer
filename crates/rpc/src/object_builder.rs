use crate::error::ClientError;
use object_builder_client::ObjectBuilderClient;
use tonic::transport::Channel;

pub use crate::codegen::object_builder::*;

pub async fn connect(addr: impl Into<String>) -> Result<ObjectBuilderClient<Channel>, ClientError> {
    ObjectBuilderClient::connect(addr.into())
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}
