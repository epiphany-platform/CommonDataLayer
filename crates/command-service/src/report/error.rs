use thiserror::Error as DeriveError;
use utils::messaging_system::Error as MSError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Failed to create producer `{0}`")]
    ProducerCreation(MSError),
    #[error("Failed to deliver Kafka report `{0}`")]
    FailedToReport(MSError),
    #[error("Failed to produce error message `{0}`")]
    FailedToProduceErrorMessage(serde_json::Error),
}
