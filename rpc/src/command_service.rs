use crate::error::ClientError;
use command_service_client::CommandServiceClient;
use tonic::transport::Channel;

tonic::include_proto!("command_service");

pub async fn connect(addr: String) -> Result<CommandServiceClient<Channel>, ClientError> {
    CommandServiceClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "command service",
            source: err,
        })
}
