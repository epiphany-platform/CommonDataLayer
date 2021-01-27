use utils::messaging_system::Error as MessagingError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to parse UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Invalid schema type. Expected `0` or `1` but found `{0}`")]
    InvalidSchemaType(i32),
    #[error("Unable to subscribe to Kafka: {0}")]
    KafkaError(MessagingError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
