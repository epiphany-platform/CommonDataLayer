use async_graphql::{InputObject, Json, SimpleObject};
use serde_json::value::RawValue;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, InputObject)]
pub struct InputMessage {
    /// Object ID
    pub object_id: Uuid,
    /// Schema ID
    pub schema_id: Uuid,
    /// JSON-encoded payload
    pub payload: Json<Box<RawValue>>,
}

#[derive(serde::Deserialize, SimpleObject)]
pub struct CdlObject {
    pub object_id: Uuid,
    pub data: Json<Value>,
}

#[derive(serde::Deserialize, SimpleObject)]
pub struct SchemaRelation {
    pub relation_id: Uuid,
    pub child_schema_id: Uuid,
    pub parent_schema_id: Uuid,
}

#[derive(serde::Deserialize, SimpleObject)]
pub struct EdgeRelations {
    pub relation_id: Uuid,
    pub parent_object_id: Uuid,
    pub child_object_ids: Vec<Uuid>,
}

#[derive(Debug, InputObject)]
pub struct ObjectRelations {
    /// Object's schema relations
    pub relation_id: Uuid,
    /// Relation parent
    pub parent_object_id: Uuid,
    /// Relation children
    pub child_object_ids: Vec<Uuid>,
}
