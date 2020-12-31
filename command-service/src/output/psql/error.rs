use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Unable to connect to server via gRPC `{0}`")]
    FailedToConnect(bb8_postgres::tokio_postgres::Error),
    #[error("Failed to parse url into postgres connection string `{0}`")]
    FailedToParseUrl(bb8_postgres::tokio_postgres::Error),
    #[error(
        "Schema `{0}` has invalid name. It can contain only ascii letters, numbers and underscores"
    )]
    InvalidSchemaName(String),
}
