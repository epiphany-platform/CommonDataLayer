use async_graphql::{Enum, InputObject, Json, SimpleObject};
use num_derive::{FromPrimitive, ToPrimitive};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug)]
/// Schema is the format in which data is to be sent to the Common Data Layer.
pub struct Schema {
    /// Random UUID assigned on creation
    pub id: Uuid,
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: String,
    /// Destination to which data is inserted by data-router.
    pub insert_destination: String,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: String,
    pub schema_type: SchemaType,
}

#[derive(Debug, Enum, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
/// Schema type, describes what kind of query service and command service is going to be used, as timeseries databases are quite different than others.
pub enum SchemaType {
    DocumentStorage = 0,
    Timeseries = 1,
}

#[derive(Debug, SimpleObject)]
/// Schema definition stores information about data structure used to push object to database.
/// Each schema can have only one active definition, under latest version but also contains history for backward compability.
pub struct Definition {
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: Json<Value>,
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist
    pub version: String,
}

/// An expression used to retrieve more complex queries
#[derive(Debug, SimpleObject)]
pub struct View {
    /// Unique identifier of view
    pub id: Uuid,
    /// The name is not required to be unique among all views (as `id` is the identifier)
    pub name: String,
    /// Materializer's address
    pub materializer_addr: String,
    /// Fields definition encoded in JSON
    pub fields: Json<Value>,
}

/// Input object which creates new schema and new definition. Each schema has to contain at least one definition, which can be later overriden.
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
    #[graphql(name = "type")]
    pub schema_type: SchemaType,
}

/// Input object which creates new view.
#[derive(Debug, Clone, InputObject)]
pub struct NewView {
    /// The name is not required to be unique among all views (as `id` is the identifier)
    pub name: String,
    /// Materializer's address
    pub materializer_addr: String,
    /// Materializer's options encoded in JSON
    pub materializer_options: Json<Value>,
    /// Fields definition encoded in JSON
    pub fields: Json<Value>,
}

/// Input object which creates new version of existing schema.
#[derive(Debug, InputObject)]
pub struct NewVersion {
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist
    /// When updating, new version has to be higher than highest stored version in DB for given schema.
    pub version: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: Json<Value>,
}

/// Input object which updates fields in schema. All fields are optional, therefore one may update only `topic` or `queryAddress` or all of them.
#[derive(Debug, InputObject)]
pub struct UpdateSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: Option<String>,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: Option<String>,
    /// Destination to which data is inserted by data-router.
    pub insert_destination: Option<String>,
    #[graphql(name = "type")]
    pub schema_type: Option<SchemaType>,
}

/// Input object which updates fields in view. All fields are optional, therefore one may update only `name` or `expression` or all of them.
#[derive(Debug, InputObject)]
pub struct UpdateView {
    /// The name is not required to be unique among all views (as `id` is the identifier)
    pub name: Option<String>,
    /// Materializer's address
    pub materializer_addr: Option<String>,
    /// Materializer's options encoded in JSON
    pub materializer_options: Option<Json<Value>>,
    /// Fields definition encoded in JSON
    pub fields: Option<Json<Value>>,
}
