use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub fields: Json<HashMap<String, FieldDefinition>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewView {
    pub schema_id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub fields: Json<HashMap<String, FieldDefinition>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewUpdate {
    pub name: Option<String>,
    pub materializer_address: Option<String>,
    pub fields: Option<Json<HashMap<String, FieldDefinition>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldDefinition {
    FieldName(String),
}
