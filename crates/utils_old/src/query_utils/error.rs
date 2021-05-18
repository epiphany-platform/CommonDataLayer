use thiserror::Error as DeriveError;

pub use tonic::{Code, Status};

#[derive(Debug, DeriveError)]
pub enum ClientError {
    #[error("Failed to connect to query service: {0}")]
    ConnectionError(tonic::transport::Error),
    #[error("Error returned from query service: {0}")]
    QueryError(Status),
}
