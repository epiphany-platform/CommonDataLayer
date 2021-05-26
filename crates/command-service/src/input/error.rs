use communication_utils::Error as MSError;
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Message payload deserialization failed: {0}")]
    PayloadDeserializationFailed(#[from] serde_json::Error),
    #[error("Failed to create message queue consumer `{0}`")]
    ConsumerCreationFailed(MSError),
    #[error("Message has a non-UUID {0}: {1}")]
    NotValidUuid(&'static str, uuid::Error),
    #[error("Message is missing payload `{0}")]
    MissingPayload(MSError),
}

impl Error {
    pub fn not_valid_order_group_id(error: uuid::Error) -> Self {
        Self::NotValidUuid("Order group ID", error)
    }
    pub fn not_valid_object_id(error: uuid::Error) -> Self {
        Self::NotValidUuid("Object ID", error)
    }
    pub fn not_valid_schema_id(error: uuid::Error) -> Self {
        Self::NotValidUuid("Schema ID", error)
    }
}
