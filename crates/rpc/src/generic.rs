use crate::error::ClientError;
use generic_rpc_client::GenericRpcClient;
use tonic::transport::Channel;

pub use crate::codegen::generic_rpc::*;

pub async fn connect(
    addr: String,
    service: &'static str,
) -> Result<GenericRpcClient<Channel>, ClientError> {
    GenericRpcClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service,
            source: err,
        })
}
