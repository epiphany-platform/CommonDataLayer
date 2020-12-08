use thiserror::Error;

pub mod consumer;
pub mod message;
pub mod publisher;

pub mod metadata_fetcher;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("Error during communication via message queue \"{0}\"")]
    InternalError(String),

    #[error("Error during joining blocking task \"{0}\"")]
    TokioError(String),
}

pub type CommunicationResult<T> = Result<T, CommunicationError>;

impl From<rdkafka::error::KafkaError> for CommunicationError {
    fn from(error: rdkafka::error::KafkaError) -> Self {
        Self::InternalError(error.to_string())
    }
}
impl From<anyhow::Error> for CommunicationError {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalError(error.to_string())
    }
}
impl From<lapin::Error> for CommunicationError {
    fn from(error: lapin::Error) -> Self {
        Self::InternalError(error.to_string())
    }
}
impl From<std::str::Utf8Error> for CommunicationError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::InternalError(error.to_string())
    }
}
impl From<tokio::task::JoinError> for CommunicationError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::TokioError(error.to_string())
    }
}
