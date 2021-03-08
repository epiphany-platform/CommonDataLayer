use std::convert::TryInto;

use juniper::FieldResult;
use semver::{Version, VersionReq};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, juniper::GraphQLEnum, Clone, Copy)]
/// Schema type, describes what kind of query service and command service is going to be used,
/// as timeseries databases are quite different than others.
pub enum SchemaType {
    DocumentStorage,
    Timeseries,
}

impl Into<SchemaType> for rpc::schema_registry::types::SchemaType {
    fn into(self) -> SchemaType {
        match self {
            rpc::schema_registry::types::SchemaType::DocumentStorage => SchemaType::DocumentStorage,
            rpc::schema_registry::types::SchemaType::Timeseries => SchemaType::Timeseries,
        }
    }
}

impl Into<rpc::schema_registry::types::SchemaType> for SchemaType {
    fn into(self) -> rpc::schema_registry::types::SchemaType {
        match self {
            SchemaType::DocumentStorage => rpc::schema_registry::types::SchemaType::DocumentStorage,
            SchemaType::Timeseries => rpc::schema_registry::types::SchemaType::Timeseries,
        }
    }
}

pub struct SchemaWithDefinitions {
    pub id: Uuid,
    pub name: String,
    pub topic_or_queue: String,
    pub query_address: String,
    pub r#type: SchemaType,
    pub definitions: Vec<Definition>,
}

impl SchemaWithDefinitions {
    pub fn get_definition(&self, version_req: VersionReq) -> Option<&Definition> {
        self.definitions
            .iter()
            .filter(|d| {
                let version = Version::parse(&d.version);
                version.map(|v| version_req.matches(&v)).unwrap_or(false)
            })
            .max_by_key(|d| &d.version)
    }

    pub fn from_rpc(schema: rpc::schema_registry::SchemaWithDefinitions) -> FieldResult<Self> {
        let r#type: rpc::schema_registry::types::SchemaType = schema.metadata.r#type.try_into()?;

        Ok(SchemaWithDefinitions {
            id: Uuid::parse_str(&schema.id)?,
            name: schema.metadata.name,
            topic_or_queue: schema.metadata.topic_or_queue,
            query_address: schema.metadata.query_address,
            r#type: r#type.into(),
            definitions: schema
                .definitions
                .into_iter()
                .map(|definition| {
                    Ok(Definition {
                        version: definition.version,
                        definition: serde_json::to_string(&serde_json::from_slice::<Value>(
                            &definition.definition,
                        )?)?,
                    })
                })
                .collect::<FieldResult<Vec<_>>>()?,
        })
    }
}

#[derive(Debug, juniper::GraphQLObject)]
/// Schema definition stores information about data structure used to push object
/// to database. Each schema can have only one active definition, under latest version
/// but also contains history for backward compability.
pub struct Definition {
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: String,
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist
    pub version: String,
}

/// Input object which creates new schema and new definition. Each schema has to contain
/// at least one definition, which can be later overriden.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct NewSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: String,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: String,
    /// Message queue topic to which data is inserted by data-router.
    pub topic_or_queue: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: String,
    /// Whether the schema stores documents or timeseries data.
    pub r#type: SchemaType,
}

/// Input object which creates new version of existing schema.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct NewVersion {
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist
    /// When updating, new version has to be higher than highest stored version in DB for given schema.
    pub version: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: String,
}

/// Input object which updates fields in schema. All fields are optional, therefore one may
/// update only `topic` or `queryAddress` or all of them.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct UpdateSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: Option<String>,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: Option<String>,
    /// Message queue topic to which data is inserted by data-router.
    pub topic_or_queue: Option<String>,
    /// Whether the schema stores documents or timeseries data.
    pub r#type: Option<SchemaType>,
}
