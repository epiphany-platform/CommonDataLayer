use num_derive::{FromPrimitive, ToPrimitive};
use uuid::Uuid;

#[derive(Debug)]
/// Schema is the format in which data is to be sent to the Common Data Layer.
pub struct Schema {
    /// Random UUID assigned on creation
    pub id: Uuid,
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: String,
    /// Kafka topic to which data is inserted by data-router.
    pub topic: String,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: String,
    pub schema_type: SchemaType,
}

#[derive(Debug, juniper::GraphQLEnum, Clone, Copy, FromPrimitive, ToPrimitive)]
/// Schema type, describes what kind of query service and command service is going to be used, as timeseries databases are quite different than others.
pub enum SchemaType {
    DocumentStorage = 0,
    Timeseries = 1,
}

#[derive(Debug, juniper::GraphQLObject)]
/// Schema definition stores information about data structure used to push object to database.
/// Each schema can have only one active definition, under latest version but also contains history for backward compability.
pub struct Definition {
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: String,
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist
    pub version: String,
}

/// An expression used to retrieve more complex queries
#[derive(Debug, juniper::GraphQLObject)]
pub struct View {
    /// Unique identifier of view
    pub id: Uuid,
    /// The name is not required to be unique among all views (as `id` is the identifier)
    pub name: String,
    /// Expression is in JMESPath format, although right now there is no validation
    pub expression: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Report {
    /// Application which generated the report
    pub application: String,
    /// Output plugin in command service
    pub output_plugin: Option<String>,
    /// Success/Failure
    pub description: String,
    /// Object id
    pub object_id: Uuid,
    /// JSON encoded payload
    pub payload: serde_json::Value,
}

/// Input object which creates new schema and new definition. Each schema has to contain at least one definition, which can be later overriden.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct NewSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: String,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: String,
    /// Kafka topic to which data is inserted by data-router.
    pub topic: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: String,
    #[graphql(name = "type")]
    pub schema_type: SchemaType,
}

/// Input object which creates new view.
#[derive(Debug, Clone, juniper::GraphQLInputObject)]
pub struct NewView {
    /// The name is not required to be unique among all views (as `id` is the identifier)
    pub name: String,
    /// Expression is in JMESPath format, although right now there is no validation
    pub expression: String,
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

/// Input object which updates fields in schema. All fields are optional, therefore one may update only `topic` or `queryAddress` or all of them.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct UpdateSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: Option<String>,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: Option<String>,
    /// Kafka topic to which data is inserted by data-router.
    pub topic: Option<String>,
    #[graphql(name = "type")]
    pub schema_type: Option<SchemaType>,
}

/// Input object which updates fields in view. All fields are optional, therefore one may update only `name` or `expression` or all of them.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct UpdateView {
    /// The name is not required to be unique among all views (as `id` is the identifier)
    pub name: Option<String>,
    /// Expression is in JMESPath format, although right now there is no validation
    pub expression: Option<String>,
}
