use crate::error::ClientError;
use general_materializer_client::GeneralMaterializerClient;
use tonic::transport::Channel;

pub use crate::codegen::materializer_general::*;

pub async fn connect(addr: String) -> Result<GeneralMaterializerClient<Channel>, ClientError> {
    GeneralMaterializerClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}
