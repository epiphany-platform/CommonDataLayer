use crate::rpc::schema::schema_type;
use indradb::{EdgeProperties, VertexProperties};
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use storage::*;

pub mod storage {
    pub mod edges;
    pub mod vertices;
}

// Helper structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub definition: Value,
    pub kafka_topic: String,
    pub query_address: String,
    pub schema_type: SchemaType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
pub enum SchemaType {
    DocumentStorage,
    Timeseries,
}

impl From<schema_type::Type> for SchemaType {
    fn from(st: schema_type::Type) -> Self {
        match st {
            schema_type::Type::DocumentStorage => SchemaType::DocumentStorage,
            schema_type::Type::Timeseries => SchemaType::Timeseries,
        }
    }
}

impl std::fmt::Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SchemaType::DocumentStorage => "DocumentStorage",
            SchemaType::Timeseries => "Timeseries",
        })
    }
}

impl std::str::FromStr for SchemaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DocumentStorage" => Ok(SchemaType::DocumentStorage),
            "Timeseries" => Ok(SchemaType::Timeseries),
            invalid => Err(anyhow::anyhow!("Invalid schema type: {}", invalid)),
        }
    }
}

impl NewSchema {
    pub fn vertex(self) -> (vertices::Schema, Value) {
        let Self {
            name,
            definition,
            kafka_topic,
            query_address,
            schema_type,
        } = self;
        (
            vertices::Schema {
                name,
                kafka_topic,
                query_address,
                schema_type,
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
    pub schemas: HashMap<Uuid, vertices::Schema>,
    pub definitions: HashMap<Uuid, vertices::Definition>,
    pub views: HashMap<Uuid, vertices::View>,
    pub schema_definitions: Vec<edges::SchemaDefinition>,
    pub schema_views: Vec<edges::SchemaView>,
}

fn extract_vertex_property<T: DeserializeOwned>(
    properties: &mut VertexProperties,
    name: &'static str,
) -> Option<T> {
    properties
        .props
        .drain_filter(|prop| prop.name == name)
        .next()
        .and_then(|prop| serde_json::from_value(prop.value).ok())
}

fn extract_edge_property<T: DeserializeOwned>(
    properties: &mut EdgeProperties,
    name: &'static str,
) -> Option<T> {
    properties
        .props
        .drain_filter(|prop| prop.name == name)
        .next()
        .and_then(|prop| serde_json::from_value(prop.value).ok())
}
