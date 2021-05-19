use crate::types::VersionedUuid;
use semver::Version;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RegistryError {
    // TODO: ensure that no sensitive info is leaked here
    #[error("Unable to connect to database: {0}")]
    ConnectionError(sqlx::Error),
    #[error("Error occurred while accessing database: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("No schema found with id \"{0}\"")]
    NoSchemaWithId(Uuid),
    #[error("No view found with id \"{0}\"")]
    NoViewWithId(Uuid),
    #[error("No insert destination found named \"{0}\"")]
    NoInsertDestination(String),
    #[error("Given schema type is invalid")]
    InvalidSchemaType,
    #[error("Invalid version retrieved from database: {0}")]
    InvalidVersion(semver::SemVerError),
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
    CacheError(String),
    #[error("{0}")]
    MQError(#[from] communication_utils::Error),
    #[error("JSON error processing view fields: {0}")]
    MalformedViewFields(serde_json::Error),
    #[error("JSON error processing view filters: {0}")]
    MalformedViewFilters(serde_json::Error),
    #[error("JSON error processing view relations: {0}")]
    MalformedViewRelations(serde_json::Error),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

fn join_with_commas<'a>(errors: impl IntoIterator<Item = &'a String>) -> String {
    errors
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl From<RegistryError> for Status {
    fn from(error: RegistryError) -> Status {
        match error {
            RegistryError::NoInsertDestination(_)
            | RegistryError::NoSchemaWithId(_)
            | RegistryError::NoViewWithId(_) => Status::not_found(error.to_string()),
            RegistryError::InvalidSchemaType
            | RegistryError::NewVersionMustBeGreatest { .. }
            | RegistryError::InvalidVersion(_)
            | RegistryError::NoVersionMatchesRequirement(_)
            | RegistryError::InvalidData(_)
            | RegistryError::InvalidJsonSchema(_) => Status::invalid_argument(error.to_string()),
            RegistryError::ConnectionError(_)
            | RegistryError::DbError(_)
            | RegistryError::MQError(_)
            | RegistryError::MalformedNotification(_)
            | RegistryError::MalformedViewFields(_)
            | RegistryError::MalformedViewFilters(_)
            | RegistryError::MalformedViewRelations(_)
            | RegistryError::NotificationError(_)
            | RegistryError::CacheError(_) => Status::internal(error.to_string()),
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
