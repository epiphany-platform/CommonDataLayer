use std::str::Utf8Error;

use crate::{communication, report};
use rdkafka::error::KafkaError;
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Failed to create Kafka consumer `{0}`")]
    ConsumerCreationFailed(KafkaError),
    #[error("Failed to subscribe to kafka topics `{0}`")]
    FailedToSubscribe(KafkaError),
    #[error("Kafka message is missing a key")]
    MissingKey,
    #[error("Kafka message has a non-UUID key: {0}")]
    KeyNotValidUuid(uuid::Error),
    #[error("Kafka message is missing a schema ID header")]
    MissingSchemaIdHeader,
    #[error("Kafka message has an invalid schema ID header")]
    InvalidSchemaIdHeader,
    #[error("Unable to parse key as valid UTF8 string `{0}`")]
    UnableToParseUTF8(Utf8Error),
    #[error("Kafka message is missing payload")]
    MissingPayload,
    #[error("Failed to read message `{0}`")]
    FailedReadingMessage(KafkaError),
    #[error("Metric is missing timestamp")]
    TimestampUnavailable,
    #[error("Failed to communicate with handler `{0}`")]
    CommunicationError(communication::Error),
    #[error("Channel was closed on sender side")]
    SenderDropped,
    #[error("Failed to initialize reporting module")]
    FailedToInitializeReporting(report::Error),
}
