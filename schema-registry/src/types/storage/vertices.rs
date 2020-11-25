use indradb::{Type, VertexProperties};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::extract_vertex_property;

pub trait Vertex: Sized {
    fn to_properties<'a>(self) -> Vec<(&'a str, Value)>;
    fn from_properties(properties: VertexProperties) -> Option<(Uuid, Self)>;
    fn db_type() -> Type;
}

lazy_static! {
    // Vertex Types
    static ref SCHEMA_VERTEX_TYPE: Type = Type::new("SCHEMA").unwrap();
    static ref SCHEMA_DEFINITION_VERTEX_TYPE: Type = Type::new("DEFINITION").unwrap();
    static ref VIEW_VERTEX_TYPE: Type = Type::new("VIEW").unwrap();
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
    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                name: extract_vertex_property(&mut properties, Self::NAME)?,
                kafka_topic: extract_vertex_property(&mut properties, Self::TOPIC_NAME)?,
                query_address: extract_vertex_property(&mut properties, Self::QUERY_ADDRESS)?,
            },
        ))
    }

    fn to_properties<'a>(self) -> Vec<(&'a str, Value)> {
        vec![
            (Self::NAME, Value::String(self.name)),
            (Self::TOPIC_NAME, Value::String(self.kafka_topic)),
            (Self::QUERY_ADDRESS, Value::String(self.query_address)),
        ]
    }

    fn db_type() -> Type {
        SCHEMA_VERTEX_TYPE.clone()
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
    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                definition: extract_vertex_property(&mut properties, Definition::VALUE)?,
            },
        ))
    }

    fn to_properties<'a>(self) -> Vec<(&'a str, Value)> {
        vec![(Definition::VALUE, self.definition)]
    }

    fn db_type() -> Type {
        SCHEMA_DEFINITION_VERTEX_TYPE.clone()
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
    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, View)> {
        Some((
            properties.vertex.id,
            View {
                name: extract_vertex_property(&mut properties, View::NAME)?,
                jmespath: extract_vertex_property(&mut properties, View::EXPRESSION)?,
            },
        ))
    }

    fn to_properties<'a>(self) -> Vec<(&'a str, Value)> {
        vec![
            (View::NAME, Value::String(self.name)),
            (View::EXPRESSION, Value::String(self.jmespath)),
        ]
    }

    fn db_type() -> Type {
        VIEW_VERTEX_TYPE.clone()
    }
}
