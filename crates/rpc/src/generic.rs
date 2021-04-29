use crate::error::ClientError;
use generic_rpc_client::GenericRpcClient;
use tonic::transport::Channel;

pub use crate::codegen::generic_rpc::*;

pub async fn connect(addr: String) -> Result<GenericRpcClient<Channel>, ClientError> {
    GenericRpcClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}
