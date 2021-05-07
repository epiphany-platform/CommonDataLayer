use thiserror::Error as DeriveError;

pub use tonic::{Code, Status};

#[derive(Debug, DeriveError)]
pub enum ClientError {
    #[error("Failed to connect to: {source}")]
    ConnectionError { source: tonic::transport::Error },
    #[error("Error durign query: {source}")]
    QueryError { source: Status },
}
