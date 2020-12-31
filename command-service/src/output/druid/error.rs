use rdkafka::error::KafkaError;
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Sender was cancelled")]
    SenderError,
    #[error("Failed sending message to kafka topic `{0}`")]
    KafkaError(KafkaError),
    #[error("Failed creating kafka producer `{0}`")]
    ProducerCreation(KafkaError),
    #[error("Data cannot be parsed `{0}`")]
    DataCannotBeParsed(serde_json::Error),
}
