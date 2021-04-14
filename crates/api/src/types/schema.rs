use std::convert::TryInto;

use async_graphql::{FieldResult, InputObject, Json, SimpleObject};
use semver::{Version, VersionReq};
use serde_json::Value;
use uuid::Uuid;

use crate::types::view::View;
use rpc::schema_registry::types::SchemaType;

pub struct FullSchema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    pub schema_type: SchemaType,
    pub definitions: Vec<Definition>,
    pub views: Vec<View>,
}

impl FullSchema {
    pub fn get_definition(&self, version_req: VersionReq) -> Option<&Definition> {
        self.definitions
            .iter()
            .filter(|d| {
                let version = Version::parse(&d.version);
                version.map(|v| version_req.matches(&v)).unwrap_or(false)
            })
            .max_by_key(|d| &d.version)
    }

    pub fn from_rpc(schema: rpc::schema_registry::FullSchema) -> FieldResult<Self> {
        let schema_type: rpc::schema_registry::types::SchemaType =
            schema.metadata.schema_type.try_into()?;

        Ok(FullSchema {
            id: Uuid::parse_str(&schema.id)?,
            name: schema.metadata.name,
            insert_destination: schema.metadata.insert_destination,
            query_address: schema.metadata.query_address,
            schema_type,
            definitions: schema
                .definitions
                .into_iter()
                .map(|definition| {
                    Ok(Definition {
                        version: definition.version,
                        definition: serde_json::from_slice(&definition.definition)?,
                    })
                })
                .collect::<FieldResult<Vec<_>>>()?,
            views: schema
                .views
                .into_iter()
                .map(View::from_rpc)
                .collect::<FieldResult<Vec<_>>>()?,
        })
    }
}

#[derive(Debug, SimpleObject)]
/// Schema definition stores information about data structure used to push object to database.
/// Each schema can have only one active definition, under latest version but also contains
/// history for backward compability.
pub struct Definition {
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: Json<Value>,
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist
    pub version: String,
}

/// Input object which creates new schema and new definition. Each schema has to
/// contain at least one definition, which can be later overriden.
#[derive(Debug, InputObject)]
pub struct NewSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: String,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: String,
    /// Destination to which data is inserted by data-router.
    pub insert_destination: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: Json<Value>,
    /// Whether the schema stores documents or timeseries data.
    #[graphql(name = "type")]
    pub schema_type: SchemaType,
}

/// Input object which creates new version of existing schema.
#[derive(Debug, InputObject)]
pub struct NewVersion {
    /// Schema is following semantic versioning, querying for "2.1.0" will
    /// return "2.1.1" if it exists. When updating, new version has to be higher
    /// than highest stored version in DB for given schema.
    pub version: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: Json<Value>,
}

/// Input object which updates fields in schema. All fields are optional,
/// therefore one may update only `topic` or `queryAddress` or all of them.
#[derive(Debug, InputObject)]
pub struct UpdateSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: Option<String>,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: Option<String>,
    /// Destination to which data is inserted by data-router.
    pub insert_destination: Option<String>,
    /// Whether the schema stores documents or timeseries data.
    #[graphql(name = "type")]
    pub schema_type: Option<SchemaType>,
}
