use crate::db::property;
use indradb::VertexProperties;
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub definition: Value,
    pub kafka_topic: String,
    pub query_address: String,
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
pub struct View {
    pub name: String,
    pub jmespath: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedUuid {
    pub id: Uuid,
    pub version_req: VersionReq,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredSchema {
    pub name: String,
    pub kafka_topic: String,
    pub query_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredDefinition {
    pub definition: Value,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DbExport {
    pub schemas: HashMap<Uuid, StoredSchema>,
    pub definitions: HashMap<Uuid, StoredDefinition>,
    pub views: HashMap<Uuid, View>,
    pub def_edges: HashMap<Uuid, Uuid>,
    pub view_edges: HashMap<Uuid, Uuid>,
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

impl View {
    pub fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, View)> {
        Some((
            properties.vertex.id,
            View {
                name: get_vertex_property_or(&mut properties, property::VIEW_NAME)?,
                jmespath: get_vertex_property_or(&mut properties, property::VIEW_EXPRESSION)?,
            },
        ))
    }
}

impl StoredDefinition {
    pub fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                definition: get_vertex_property_or(&mut properties, property::DEFINITION_VALUE)?,
            },
        ))
    }
}

impl StoredSchema {
    pub fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, Self)> {
        Some((
            properties.vertex.id,
            Self {
                name: get_vertex_property_or(&mut properties, property::SCHEMA_NAME)?,
                kafka_topic: get_vertex_property_or(&mut properties, property::SCHEMA_TOPIC_NAME)?,
                query_address: get_vertex_property_or(
                    &mut properties,
                    property::SCHEMA_QUERY_ADDRESS,
                )?,
            },
        ))
    }
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
