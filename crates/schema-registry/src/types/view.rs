use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgValueRef, types::Json, Decode, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub id: Uuid,
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

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct FieldDefinitions {
//     pub fields: HashMap<String, FieldDefinition>,
// }

// impl<'r> Decode<'r, Postgres> for FieldDefinitions {
//     fn decode(
//         value: PgValueRef<'r>,
//     ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
//         let json = <Json<Value> as Decode<Postgres>>::decode(value)?;

//         serde_json::from_value(json.0).map_err(Into::into)
//     }
// }
