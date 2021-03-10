use crate::report;
use thiserror::Error as DeriveError;
use utils::communication::Error as MSError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Message payload deserialization failed: {0}")]
    PayloadDeserializationFailed(#[from] serde_json::Error),
    #[error("Failed to create message queue consumer `{0}`")]
    ConsumerCreationFailed(MSError),
    #[error("Failed to acknowledge message `{0}`")]
    FailedToAcknowledge(MSError),
    #[error("Failed to create GRPC server `{0}`")]
    ServerCreationFailed(#[from] tonic::transport::Error),
    #[error("Message has a non-UUID {0}: {1}")]
    NotValidUuid(&'static str, uuid::Error),
    #[error("Message is missing payload `{0}")]
    MissingPayload(MSError),
    #[error("Failed to read message `{0}`")]
    FailedReadingMessage(MSError),
    #[error("Failed to communicate with handler `{0}`")]
    CommunicationError(report::Error),
    #[error("Failed to initialize reporting module")]
    FailedToInitializeReporting(report::Error),
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
