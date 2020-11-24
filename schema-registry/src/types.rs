use indradb::{EdgeKey, EdgeProperties, Type, VertexProperties};
use lazy_static::lazy_static;
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

lazy_static! {
    // Vertex Types
    pub static ref SCHEMA_VERTEX_TYPE: Type = Type::new("SCHEMA").unwrap();
    pub static ref SCHEMA_DEFINITION_VERTEX_TYPE: Type = Type::new("DEFINITION").unwrap();
    pub static ref VIEW_VERTEX_TYPE: Type = Type::new("VIEW").unwrap();

    // Edge Types
    pub static ref SCHEMA_DEFINITION_EDGE_TYPE: Type = Type::new("SCHEMA_DEFINITION").unwrap();
    pub static ref SCHEMA_VIEW_EDGE_TYPE: Type = Type::new("SCHEMA_VIEW").unwrap();
}

pub trait Vertex: Sized {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>);
    fn from_properties(properties: VertexProperties) -> Option<(Uuid, Self)>;
}

pub trait Edge: Sized {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>);
    fn from_properties(properties: EdgeProperties) -> Option<Self>;
}

// Stored vertices
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    pub name: String,
    pub kafka_topic: String,
    pub query_address: String,
}

impl Schema {
    pub const NAME: &'static str = "SCHEMA_NAME";
    pub const TOPIC_NAME: &'static str = "SCHEMA_TOPIC_NAME";
    pub const QUERY_ADDRESS: &'static str = "SCHEMA_QUERY_ADDRESS";
}

impl Vertex for Schema {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>) {
        (
            SCHEMA_VERTEX_TYPE.clone(),
            vec![
                (Self::NAME, Value::String(self.name)),
                (Self::TOPIC_NAME, Value::String(self.kafka_topic)),
                (Self::QUERY_ADDRESS, Value::String(self.query_address)),
            ],
        )
    }

    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                name: get_vertex_property_or(&mut properties, Self::NAME)?,
                kafka_topic: get_vertex_property_or(&mut properties, Self::TOPIC_NAME)?,
                query_address: get_vertex_property_or(&mut properties, Self::QUERY_ADDRESS)?,
            },
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Definition {
    pub definition: Value,
}

impl Definition {
    pub const VALUE: &'static str = "DEFINITION";
}

impl Vertex for Definition {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>) {
        (
            SCHEMA_DEFINITION_VERTEX_TYPE.clone(),
            vec![(Definition::VALUE, self.definition)],
        )
    }

    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                definition: get_vertex_property_or(&mut properties, Definition::VALUE)?,
            },
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct View {
    pub name: String,
    pub jmespath: String,
}

impl View {
    pub const NAME: &'static str = "VIEW_NAME";
    pub const EXPRESSION: &'static str = "JMESPATH";
}

impl Vertex for View {
    fn vertex_info<'a>(self) -> (Type, Vec<(&'a str, Value)>) {
        (
            VIEW_VERTEX_TYPE.clone(),
            vec![
                (View::NAME, Value::String(self.name)),
                (View::EXPRESSION, Value::String(self.jmespath)),
            ],
        )
    }

    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, View)> {
        Some((
            properties.vertex.id,
            View {
                name: get_vertex_property_or(&mut properties, View::NAME)?,
                jmespath: get_vertex_property_or(&mut properties, View::EXPRESSION)?,
            },
        ))
    }
}

// Stored edges
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ViewEdge {
    pub schema_id: Uuid,
    pub view_id: Uuid,
}

impl Edge for ViewEdge {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>) {
        (
            EdgeKey::new(self.schema_id, SCHEMA_VIEW_EDGE_TYPE.clone(), self.view_id),
            vec![],
        )
    }

    fn from_properties(properties: EdgeProperties) -> Option<Self> {
        let schema_id = properties.edge.key.outbound_id;
        let view_id = properties.edge.key.inbound_id;
        Some(Self { schema_id, view_id })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DefinitionEdge {
    pub schema_id: Uuid,
    pub def_id: Uuid,
    pub version: Version,
}

impl DefinitionEdge {
    pub const VERSION: &'static str = "VERSION";
}

impl Edge for DefinitionEdge {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>) {
        (
            EdgeKey::new(
                self.schema_id,
                SCHEMA_DEFINITION_EDGE_TYPE.clone(),
                self.def_id,
            ),
            vec![(Self::VERSION, serde_json::json!(self.version))],
        )
    }

    fn from_properties(mut properties: EdgeProperties) -> Option<Self> {
        let schema_id = properties.edge.key.outbound_id;
        let def_id = properties.edge.key.inbound_id;
        Some(Self {
            schema_id,
            def_id,
            version: get_edge_property_or(&mut properties, Self::VERSION)?,
        })
    }
}

// Helper structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub definition: Value,
    pub kafka_topic: String,
    pub query_address: String,
}

impl NewSchema {
    pub fn vertex(self) -> (Schema, Value) {
        let Self {
            name,
            definition,
            kafka_topic,
            query_address,
        } = self;
        (
            Schema {
                name,
                kafka_topic,
                query_address,
            },
            definition,
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchemaVersion {
    pub version: Version,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub version: Version,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedUuid {
    pub id: Uuid,
    pub version_req: VersionReq,
}

impl VersionedUuid {
    pub fn new(id: Uuid, version_req: VersionReq) -> Self {
        Self { id, version_req }
    }

    pub fn exact(id: Uuid, version: Version) -> Self {
        Self {
            id,
            version_req: VersionReq::exact(&version),
        }
    }

    pub fn any(id: Uuid) -> Self {
        Self {
            id,
            version_req: VersionReq::any(),
        }
    }
}

// Import export
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DbExport {
    pub schemas: HashMap<Uuid, Schema>,
    pub definitions: HashMap<Uuid, Definition>,
    pub views: HashMap<Uuid, View>,
    pub def_edges: Vec<DefinitionEdge>,
    pub view_edges: Vec<ViewEdge>,
}

fn get_vertex_property_or<T: DeserializeOwned>(
    properties: &mut VertexProperties,
    name: &'static str,
) -> Option<T> {
    properties
        .props
        .drain_filter(|prop| prop.name == name)
        .next()
        .and_then(|prop| serde_json::from_value(prop.value).ok())
}

fn get_edge_property_or<T: DeserializeOwned>(
    properties: &mut EdgeProperties,
    name: &'static str,
) -> Option<T> {
    properties
        .props
        .drain_filter(|prop| prop.name == name)
        .next()
        .and_then(|prop| serde_json::from_value(prop.value).ok())
}
