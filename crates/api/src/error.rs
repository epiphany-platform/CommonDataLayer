use communication_utils::Error as MessagingError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to parse UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Invalid schema type. Expected `0` or `1` but found `{0}`")]
    InvalidSchemaType(i32),
    #[error("Unable to connect to data router publisher: {0}")]
    PublisherError(MessagingError),
    #[error("Error while parsing view fields: {0}")]
    ViewFieldError(serde_json::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
