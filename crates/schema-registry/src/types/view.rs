use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use utils::types::FieldDefinition;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub materializer_options: Value,
    pub fields: Json<HashMap<String, FieldDefinition>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewView {
    pub schema_id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub materializer_options: Value,
    pub fields: Json<HashMap<String, FieldDefinition>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewUpdate {
    pub name: Option<String>,
    pub materializer_address: Option<String>,
    pub materializer_options: Option<Value>,
    pub fields: Option<Json<HashMap<String, FieldDefinition>>>,
}
