use crate::types::VersionedUuid;
use rpc::error::ClientError;
use semver::Version;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("No schema found with id \"{0}\"")]
    NoSchemaWithId(Uuid),
    #[error("No view found with id \"{0}\"")]
    NoViewWithId(Uuid),
    #[error("Object with id \"{0}\" already exists in database")]
    DuplicatedUuid(Uuid),
    #[error("Error occurred while accessing sled database: {0}")]
    DbError(indradb::Error),
    #[error("Unable to connect to sled database: {0}")]
    ConnectionError(indradb::Error),
    #[error("No topic found named \"{0}\"")]
    NoTopic(String),
    #[error("{0}")]
    MQError(String),
    #[error("Given schema was invalid")]
    InvalidSchema,
    #[error("Given schema type is invalid")]
    InvalidSchemaType,
    #[error("Given view was invalid: {0}")]
    InvalidView(String),
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
    #[error("{0}")]
    CacheError(String),
    #[error("{0}")]
    MalformedError(MalformedError),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

impl From<indradb::Error> for RegistryError {
    fn from(error: indradb::Error) -> RegistryError {
        RegistryError::DbError(error)
    }
}

impl From<jsonschema::CompilationError> for RegistryError {
    fn from(_error: jsonschema::CompilationError) -> RegistryError {
        RegistryError::InvalidSchema
    }
}

impl From<utils::messaging_system::Error> for RegistryError {
    fn from(error: utils::messaging_system::Error) -> Self {
        Self::MQError(error.to_string())
    }
}

impl From<RegistryError> for Status {
    fn from(error: RegistryError) -> Status {
        match error {
            RegistryError::NoTopic(_)
            | RegistryError::NoSchemaWithId(_)
            | RegistryError::NoViewWithId(_) => Status::not_found(error.to_string()),
            RegistryError::NewVersionMustBeGreatest { .. }
            | RegistryError::InvalidSchema
            | RegistryError::InvalidSchemaType
            | RegistryError::DuplicatedUuid(_)
            | RegistryError::DbError(_)
            | RegistryError::ConnectionError(_)
            | RegistryError::MQError(_)
            | RegistryError::InvalidView(_)
            | RegistryError::NoVersionMatchesRequirement(_)
            | RegistryError::CacheError(_)
            | RegistryError::MalformedError(_) => Status::internal(error.to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum MalformedError {
    #[error("Schema in database was malformed with id {0}")]
    MalformedSchema(Uuid),
    #[error("Schema definition in database was malformed with id {0}")]
    MalformedDefinition(Uuid),
    #[error("Schema version in database was malformed under schema with id {0}")]
    MalformedSchemaVersion(Uuid),
    #[error("View in database was malformed for view with id {0}")]
    MalformedView(Uuid),
}

impl From<MalformedError> for RegistryError {
    fn from(error: MalformedError) -> RegistryError {
        RegistryError::MalformedError(error)
    }
}

#[derive(Debug, Error)]
pub enum RegistryClientError {
    #[error("Failed to connect to the schema registry: {0}")]
    ConnectionError(tonic::transport::Error),
    #[error("Schema returned from registry was not valid JSON: {0}")]
    ParseSchemaError(String),
    #[error("Invalid schema returned from registry")]
    InvalidSchema,
    #[error("Input data does not match schema: {}", join_with_commas(.0))]
    InvalidData(Vec<String>),
    #[error("{}", .0.message())]
    RequestFailure(Status),
    #[error("JMESPath encountered an error: {0}")]
    JmespathError(jmespatch::JmespathError),
    #[error("JMESPath returned malformed JSON")]
    JmespathReturnedMalformedJson,
    #[error("Error occurred during marshalling of JSON value: {0}")]
    JsonError(#[source] serde_json::Error),
}

fn join_with_commas<'a>(errors: impl IntoIterator<Item = &'a String>) -> String {
    errors
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl From<ClientError> for RegistryClientError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::ConnectionError { source, .. } => {
                RegistryClientError::ConnectionError(source)
            }
            ClientError::QueryError { source, .. } => RegistryClientError::RequestFailure(source),
        }
    }
}

impl From<Status> for RegistryClientError {
    fn from(error: Status) -> Self {
        RegistryClientError::RequestFailure(error)
    }
}

impl From<RegistryClientError> for Status {
    fn from(error: RegistryClientError) -> Status {
        Status::internal(error.to_string())
    }
}
