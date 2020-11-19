use thiserror::Error as DeriveError;
use utils::messaging_system::CommunicationError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Failed to create producer `{0}`")]
    ProducerCreation(CommunicationError),
    #[error("Channel was closed on sender side.")]
    SenderDropped,
    #[error("Failed to deliver Kafka report")]
    FailedToReport(CommunicationError),
    #[error("Failed to produce error message `{0}`")]
    FailedToProduceErrorMessage(serde_json::Error),
}
