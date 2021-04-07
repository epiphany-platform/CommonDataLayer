use std::collections::HashMap;

use indradb::{Type, VertexProperties};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::{extract_vertex_property, SchemaType};
pub use utils::types::FieldDefinition;

pub trait Vertex: Sized {
    fn into_properties<'a>(self) -> Vec<(&'a str, Value)>;
    fn from_properties(properties: VertexProperties) -> Option<(Uuid, Self)>;
    fn db_type() -> Type;
}

lazy_static! {
    // Vertex Types
    static ref SCHEMA_VERTEX_TYPE: Type = Type::new("SCHEMA").unwrap();
    static ref SCHEMA_DEFINITION_VERTEX_TYPE: Type = Type::new("DEFINITION").unwrap();
    static ref VIEW_VERTEX_TYPE: Type = Type::new("VIEW").unwrap();
    static ref FIELD_DEFINITION_VERTEX_TYPE: Type = Type::new("FIELD").unwrap();
}

// Stored vertices
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    pub schema_type: SchemaType,
}

impl Schema {
    pub const NAME: &'static str = "SCHEMA_NAME";
    pub const INSERT_DESTINATION: &'static str = "SCHEMA_INSERT_DESTINATION";
    pub const QUERY_ADDRESS: &'static str = "SCHEMA_QUERY_ADDRESS";
    pub const SCHEMA_TYPE: &'static str = "SCHEMA_TYPE";
}

impl Vertex for Schema {
    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                name: extract_vertex_property(&mut properties, Self::NAME)?,
                insert_destination: extract_vertex_property(
                    &mut properties,
                    Self::INSERT_DESTINATION,
                )?,
                query_address: extract_vertex_property(&mut properties, Self::QUERY_ADDRESS)?,
                schema_type: extract_vertex_property(&mut properties, Self::SCHEMA_TYPE)?,
            },
        ))
    }

    fn into_properties<'a>(self) -> Vec<(&'a str, Value)> {
        vec![
            (Self::NAME, Value::String(self.name)),
            (
                Self::INSERT_DESTINATION,
                Value::String(self.insert_destination),
            ),
            (Self::QUERY_ADDRESS, Value::String(self.query_address)),
            (
                Self::SCHEMA_TYPE,
                Value::String(self.schema_type.to_string()),
            ),
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

    fn into_properties<'a>(self) -> Vec<(&'a str, Value)> {
        vec![(Definition::VALUE, self.definition)]
    }

    fn db_type() -> Type {
        SCHEMA_DEFINITION_VERTEX_TYPE.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct View {
    pub name: String,
    pub materializer_addr: String,
    pub fields: HashMap<String, FieldDefinition>,
}

impl View {
    pub const NAME: &'static str = "VIEW_NAME";
    pub const MATERIALIZER_ADDR: &'static str = "MATERIALIZER_ADDR";
    pub const FIELDS: &'static str = "FIELDS";
}

impl Vertex for View {
    fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, View)> {
        Some((
            properties.vertex.id,
            View {
                name: extract_vertex_property(&mut properties, View::NAME)?,
                materializer_addr: extract_vertex_property(
                    &mut properties,
                    View::MATERIALIZER_ADDR,
                )?,
                fields: extract_vertex_property(&mut properties, View::FIELDS)?,
            },
        ))
    }

    fn into_properties<'a>(self) -> Vec<(&'a str, Value)> {
        vec![
            (View::NAME, Value::String(self.name)),
            (
                View::MATERIALIZER_ADDR,
                Value::String(self.materializer_addr),
            ),
            (View::FIELDS, serde_json::to_value(&self.fields).unwrap()),
        ]
    }

    fn db_type() -> Type {
        VIEW_VERTEX_TYPE.clone()
    }
}
