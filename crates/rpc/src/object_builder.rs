use crate::error::ClientError;
use object_builder_client::ObjectBuilderClient;
use tonic::transport::Channel;

tonic::include_proto!("object_builder");

pub async fn connect(addr: String) -> Result<ObjectBuilderClient<Channel>, ClientError> {
    ObjectBuilderClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "object builder",
            source: err,
        })
}
