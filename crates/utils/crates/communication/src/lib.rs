#![feature(linked_list_cursors)]

#[cfg(any(feature = "kafka", feature = "amqp"))]
pub mod consumer;
#[cfg(feature = "kafka")]
mod kafka_ack_queue;
pub mod message;
#[cfg(any(feature = "kafka", feature = "amqp", feature = "grpc"))]
pub mod metadata_fetcher;
#[cfg(any(feature = "kafka", feature = "amqp", feature = "grpc"))]
pub mod parallel_consumer;
#[cfg(any(
    feature = "kafka",
    feature = "amqp",
    feature = "grpc",
    feature = "http"
))]
pub mod publisher;

use message::CommunicationMessage;
use thiserror::Error as DeriveError;

#[derive(Clone, Debug, DeriveError)]
pub enum Error {
    #[error("Error during communication \"{0}\"")]
    CommunicationError(String),

    #[error("Error during joining blocking task \"{0}\"")]
    RuntimeError(String),

    #[cfg(feature = "grpc")]
    #[error("GRPC server returned status: {0}")]
    GrpcStatusCode(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(feature = "grpc")]
impl From<tonic::transport::Error> for Error {
    fn from(error: tonic::transport::Error) -> Self {
        Self::CommunicationError(error.to_string())
    }
}

#[cfg(feature = "kafka")]
impl From<rdkafka::error::KafkaError> for Error {
    fn from(error: rdkafka::error::KafkaError) -> Self {
        Self::CommunicationError(error.to_string())
    }
}
impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::CommunicationError(error.to_string())
    }
}
#[cfg(feature = "amqp")]
impl From<lapin::Error> for Error {
    fn from(error: lapin::Error) -> Self {
        Self::CommunicationError(error.to_string())
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::CommunicationError(error.to_string())
    }
}
impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::RuntimeError(error.to_string())
    }
}
#[cfg(feature = "http")]
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::CommunicationError(error.to_string())
    }
}

#[cfg(feature = "grpc")]
impl From<rpc::error::ClientError> for Error {
    fn from(error: rpc::error::ClientError) -> Self {
        Self::CommunicationError(error.to_string())
    }
}

pub fn get_order_group_id(message: &dyn CommunicationMessage) -> Option<String> {
    message
        .key()
        .ok()
        .filter(|x| !x.is_empty())
        .map(|x| x.to_owned())
}
