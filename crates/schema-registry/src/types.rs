use indradb::{EdgeProperties, VertexProperties};
use rpc::schema_registry::types::SchemaType;
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
pub struct ViewUpdate {
    pub name: Option<String>,
    pub jmespath: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub definition: Value,
    pub kafka_topic: String,
    pub query_address: String,
    pub schema_type: SchemaType,
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
