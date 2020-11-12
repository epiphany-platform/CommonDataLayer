use thiserror::Error;

pub mod consumer;
pub mod message;
pub mod publisher;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("Error during communication via message queue \"{0}\"")]
    InternalError(String),
}

pub type CommunicationResult<T> = Result<T, CommunicationError>;

impl From<rdkafka::error::KafkaError> for CommunicationError {
    fn from(error: rdkafka::error::KafkaError) -> CommunicationError {
        CommunicationError::InternalError(error.to_string())
    }
}
impl From<anyhow::Error> for CommunicationError {
    fn from(error: anyhow::Error) -> CommunicationError {
        CommunicationError::InternalError(error.to_string())
    }
}
impl From<lapin::Error> for CommunicationError {
    fn from(error: lapin::Error) -> CommunicationError {
        CommunicationError::InternalError(error.to_string())
    }
}
impl From<std::str::Utf8Error> for CommunicationError {
    fn from(error: std::str::Utf8Error) -> CommunicationError {
        CommunicationError::InternalError(error.to_string())
    }
}
