use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use rpc::schema_registry::types::SchemaType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    pub r#type: SchemaType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    pub r#type: SchemaType,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaUpdate {
    pub name: Option<String>,
    pub insert_destination: Option<String>,
    pub query_address: Option<String>,
    pub r#type: Option<SchemaType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaWithDefinitions {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    pub r#type: SchemaType,
    pub definitions: Vec<SchemaDefinition>,
}

impl SchemaWithDefinitions {
    pub fn definition(&self, version: VersionReq) -> Option<&SchemaDefinition> {
        self.definitions
            .iter()
            .filter(|d| version.matches(&d.version))
            .max_by_key(|d| &d.version)
    }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbExport {
    pub schemas: Vec<SchemaWithDefinitions>,
}
