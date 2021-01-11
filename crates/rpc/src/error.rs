use thiserror::Error as DeriveError;

pub use tonic::{Code, Status};

#[derive(Debug, DeriveError)]
pub enum ClientError {
    #[error("Failed to connect to {service}: {source}")]
    ConnectionError {
        service: &'static str,
        source: tonic::transport::Error,
    },
    #[error("Error returned from {service}: {source}")]
    QueryError {
        service: &'static str,
        source: Status,
    },
}

pub fn registry_error(error: Status) -> ClientError {
    ClientError::QueryError {
        service: "schema registry",
        source: error,
    }
}
