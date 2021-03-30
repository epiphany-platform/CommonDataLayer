use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::view::View;
use rpc::schema_registry::types::SchemaType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaUpdate {
    pub name: Option<String>,
    pub insert_destination: Option<String>,
    pub query_address: Option<String>,
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullSchema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definitions: Vec<SchemaDefinition>,
    pub views: Vec<View>,
}

impl FullSchema {
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
