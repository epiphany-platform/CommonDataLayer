use rdkafka::error::KafkaError;
use thiserror::Error as DeriveError;
use serde_json;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Sender was cancelled")]
    SenderError,
    #[error("Failed sending message to kafka topic `{0}`")]
    KafkaError(KafkaError),
    #[error("Failed creating kafka producer `{0}`")]
    ProducerCreation(KafkaError),
    #[error("Channel was closed on receiver side.")]
    RecvDropped,
    #[error("Data cannot be parsed `{0}`")]
    DataCannotBeParsed(serde_json::Error),
}
