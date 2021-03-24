use async_graphql::{Json, SimpleObject};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, SimpleObject)]
pub struct Report {
    /// Application which generated the report
    pub application: String,
    /// Output plugin in command service
    pub output_plugin: Option<String>,
    /// Success/Failure
    pub description: String,
    /// Object id
    pub object_id: Uuid,
    /// JSON encoded payload
    pub payload: Json<Value>,
}
