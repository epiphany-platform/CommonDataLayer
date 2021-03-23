use crate::types::VersionedUuid;
use semver::Version;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Unable to connect to database: {0}")]
    ConnectionError(sqlx::Error),
    #[error("Error occurred while accessing database: {0}")]
    DbError(sqlx::Error),
    #[error("No schema found with id \"{0}\"")]
    NoSchemaWithId(Uuid),
    #[error("Given schema type is invalid")]
    InvalidSchemaType,
    #[error("Invalid version retrieved from database: {0}")]
    InvalidVersion(semver::SemVerError),
    #[error("No topic found named \"{0}\"")]
    NoTopic(String),
    #[error("No version of schema with id {} matches the given requirement {}", .0.id, .0.version_req)]
    NoVersionMatchesRequirement(VersionedUuid),
    #[error(
        "New schema version for schema with id {schema_id} \
         must be greater than the current max {max_version}"
    )]
    NewVersionMustBeGreatest {
        schema_id: Uuid,
        max_version: Version,
    },
    #[error("Input data does not match schema: {}", join_with_commas(.0))]
    InvalidData(Vec<String>),
    #[error("Invalid JSON schema: {0}")]
    InvalidJsonSchema(jsonschema::CompilationError),
    #[error("Error receiving notification from database: {0}")]
    NotificationError(sqlx::Error),
    #[error("Malformed notification payload: {0}")]
    MalformedNotification(serde_json::Error),
    #[error("{0}")]
    MQError(utils::communication::Error),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

fn join_with_commas<'a>(errors: impl IntoIterator<Item = &'a String>) -> String {
    errors
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl From<sqlx::Error> for RegistryError {
    fn from(error: sqlx::Error) -> Self {
        RegistryError::DbError(error)
    }
}

impl From<utils::communication::Error> for RegistryError {
    fn from(error: utils::communication::Error) -> Self {
        Self::MQError(error)
    }
}

impl From<RegistryError> for Status {
    fn from(error: RegistryError) -> Status {
        match error {
            RegistryError::NoSchemaWithId(_) => Status::not_found(error.to_string()),
            RegistryError::InvalidSchemaType
            | RegistryError::NewVersionMustBeGreatest { .. }
            | RegistryError::InvalidVersion(_)
            | RegistryError::NoVersionMatchesRequirement(_)
            | RegistryError::InvalidData(_)
            | RegistryError::NoTopic(_)
            | RegistryError::InvalidJsonSchema(_) => Status::invalid_argument(error.to_string()),
            RegistryError::ConnectionError(_)
            | RegistryError::DbError(_)
            | RegistryError::MQError(_)
            | RegistryError::NotificationError(_)
            | RegistryError::MalformedNotification(_) => Status::internal(error.to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to connect to schema registry: {0}")]
    ConnectionError(rpc::error::ClientError),
    #[error("Error returned from schema registry: {0}")]
    RegistryError(tonic::Status),
    #[error("Missing schema")]
    MissingSchema,
    #[error("Malformed schema")]
    MalformedSchema,
    #[error("Failed to receive schema update: {0}")]
    SchemaUpdateReceiveError(tonic::Status),
}

pub type CacheResult<T> = Result<T, CacheError>;
