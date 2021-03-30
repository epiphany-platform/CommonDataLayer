use async_graphql::{InputObject, Json, SimpleObject};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, InputObject)]
pub struct InputMessage {
    /// Object ID
    pub object_id: Uuid,
    /// Schema ID
    pub schema_id: Uuid,
    /// JSON-encoded payload
    pub payload: Json<Value>,
}

#[derive(serde::Deserialize, SimpleObject)]
pub struct CdlObject {
    pub object_id: Uuid,
    pub data: Json<Value>,
}
