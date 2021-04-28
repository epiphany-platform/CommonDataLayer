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

impl ObjectRelations {
    pub fn into_edge_rpc(self) -> rpc::edge_registry::Edge {
        rpc::edge_registry::Edge {
            relation_id: self.relation_id.to_string(),
            parent_object_id: self.parent_object_id.to_string(),
            child_object_ids: self
                .child_object_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}
