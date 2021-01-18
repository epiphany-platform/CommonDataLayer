use indradb::{EdgeKey, EdgeProperties, Type};
use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::extract_edge_property;

pub trait Edge: Sized {
    fn db_type() -> Type;
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>);
    fn from_properties(properties: EdgeProperties) -> Option<Self>;
}

lazy_static! {
    // Edge Types
    static ref SCHEMA_DEFINITION_EDGE_TYPE: Type = Type::new("SCHEMA_DEFINITION").unwrap();
    static ref SCHEMA_VIEW_EDGE_TYPE: Type = Type::new("SCHEMA_VIEW").unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SchemaView {
    pub schema_id: Uuid,
    pub view_id: Uuid,
}

impl Edge for SchemaView {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>) {
        (
            EdgeKey::new(self.schema_id, Self::db_type(), self.view_id),
            vec![],
        )
    }

    fn from_properties(properties: EdgeProperties) -> Option<Self> {
        let schema_id = properties.edge.key.outbound_id;
        let view_id = properties.edge.key.inbound_id;
        Some(Self { schema_id, view_id })
    }

    fn db_type() -> Type {
        SCHEMA_VIEW_EDGE_TYPE.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SchemaDefinition {
    pub schema_id: Uuid,
    pub definition_id: Uuid,
    pub version: Version,
}

impl SchemaDefinition {
    pub const VERSION: &'static str = "VERSION";
}

impl Edge for SchemaDefinition {
    fn edge_info<'a>(self) -> (EdgeKey, Vec<(&'a str, Value)>) {
        (
            EdgeKey::new(self.schema_id, Self::db_type(), self.definition_id),
            vec![(Self::VERSION, serde_json::json!(self.version))],
        )
    }

    fn from_properties(mut properties: EdgeProperties) -> Option<Self> {
        let schema_id = properties.edge.key.outbound_id;
        let definition_id = properties.edge.key.inbound_id;
        Some(Self {
            schema_id,
            definition_id,
            version: extract_edge_property(&mut properties, Self::VERSION)?,
        })
    }

    fn db_type() -> Type {
        SCHEMA_DEFINITION_EDGE_TYPE.clone()
    }
}
