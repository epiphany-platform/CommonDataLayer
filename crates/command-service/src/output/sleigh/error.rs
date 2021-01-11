use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Unable to connect to server via gRPC `{0}`")]
    FailedToConnect(tonic::transport::Error),
}
