use juniper::{graphql_object, GraphQLInputObject};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, GraphQLInputObject)]
pub struct InputMessage {
    /// Object ID
    pub object_id: Uuid,
    /// Schema ID
    pub schema_id: Uuid,
    /// JSON-encoded payload
    pub payload: String,
}

#[derive(serde::Deserialize)]
pub struct CdlObject {
    pub object_id: Uuid,
    pub data: Value,
}

#[graphql_object]
impl CdlObject {
    /// The Object ID
    pub fn object_id(&self) -> Uuid {
        self.object_id
    }

    /// The Payload of the Object
    pub fn data(&self) -> String {
        serde_json::to_string(&self.data).unwrap_or_default()
    }
}
