use crate::report;
use std::str::Utf8Error;
use thiserror::Error as DeriveError;
use utils::messaging_system::Error as MSError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Message payload deserialization failed: {0}")]
    PayloadDeserializationFailed(#[from] serde_json::Error),
    #[error("Failed to create message queue consumer `{0}`")]
    ConsumerCreationFailed(MSError),
    #[error("Failed to subscribe to: `{0}`")]
    FailedToSubscribe(MSError),
    #[error("Failed to acknowledge message `{0}`")]
    FailedToAcknowledge(MSError),
    #[error("Message is missing a key `{0}`")]
    MissingKey(MSError),
    #[error("Failed to create GRPC server `{0}`")]
    ServerCreationFailed(#[from] tonic::transport::Error),
    #[error("Message has a non-UUID key: {0}")]
    KeyNotValidUuid(uuid::Error),
    #[error("Message is missing a schema ID header")]
    MissingSchemaIdHeader,
    #[error("Message has an invalid schema ID header")]
    InvalidSchemaIdHeader,
    #[error("Unable to parse key as valid UTF8 string `{0}`")]
    UnableToParseUTF8(Utf8Error),
    #[error("Message is missing payload `{0}")]
    MissingPayload(MSError),
    #[error("Failed to read message `{0}`")]
    FailedReadingMessage(MSError),
    #[error("Metric is missing timestamp")]
    TimestampUnavailable,
    #[error("Failed to communicate with handler `{0}`")]
    CommunicationError(report::Error),
    #[error("Failed to initialize reporting module")]
    FailedToInitializeReporting(report::Error),
}
