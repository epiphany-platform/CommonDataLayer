use std::collections::HashMap;

use cdl_dto::materialization::{FieldDefinition, Filter, Relation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullView {
    pub id: Uuid,
    pub base_schema: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub materializer_options: Value,
    pub fields: Json<HashMap<String, FieldDefinition>>,
    #[serde(default)]
    pub relations: Json<Vec<Relation>>,
    #[serde(default)]
    pub filters: Json<Option<Filter>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub materializer_options: Value,
    pub fields: Json<HashMap<String, FieldDefinition>>,
    #[serde(default)]
    pub relations: Json<Vec<Relation>>,
    #[serde(default)]
    pub filters: Json<Option<Filter>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewView {
    pub base_schema_id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub materializer_options: Value,
    pub fields: Json<HashMap<String, FieldDefinition>>,
    #[serde(default)]
    pub relations: Json<Vec<Relation>>,
    #[serde(default)]
    pub filters: Json<Option<Filter>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewUpdate {
    pub name: Option<String>,
    pub materializer_address: Option<String>,
    pub materializer_options: Option<Value>,
    pub fields: Option<Json<HashMap<String, FieldDefinition>>>,
    pub relations: Option<Json<Vec<Relation>>>,
    pub filters: Option<Json<Option<Filter>>>,
}
