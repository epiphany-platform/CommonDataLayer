use crate::{output, report};
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Failed to insert data into database `{0}`")]
    FailedToInsertData(output::OutputError),
    #[error("Failed to send failure report `{0}`")]
    ReportingError(report::Error),
}
