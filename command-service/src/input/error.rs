use crate::report;
use std::str::Utf8Error;
use thiserror::Error as DeriveError;
use utils::messaging_system::CommunicationError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Message payload deserialization failed: {0}")]
    PayloadDeserializationFailed(#[from] serde_json::Error),
    #[error("Failed to create Kafka consumer `{0}`")]
    ConsumerCreationFailed(CommunicationError),
    #[error("Failed to subscribe to kafka topics `{0}`")]
    FailedToSubscribe(CommunicationError),
    #[error("Kafka message is missing a key `{0}`")]
    MissingKey(CommunicationError),
    #[error("Kafka message has a non-UUID key: {0}")]
    KeyNotValidUuid(uuid::Error),
    #[error("Kafka message is missing a schema ID header")]
    MissingSchemaIdHeader,
    #[error("Kafka message has an invalid schema ID header")]
    InvalidSchemaIdHeader,
    #[error("Unable to parse key as valid UTF8 string `{0}`")]
    UnableToParseUTF8(Utf8Error),
    #[error("Kafka message is missing payload `{0}")]
    MissingPayload(CommunicationError),
    #[error("Failed to read message `{0}`")]
    FailedReadingMessage(CommunicationError),
    #[error("Metric is missing timestamp")]
    TimestampUnavailable,
    #[error("Failed to communicate with handler `{0}`")]
    CommunicationError(report::Error),
    #[error("Channel was closed on sender side")]
    SenderDropped,
    #[error("Failed to initialize reporting module")]
    FailedToInitializeReporting(report::Error),
}
