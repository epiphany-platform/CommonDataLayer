use crate::schema::context::Context;
use juniper::{graphql_object, FieldResult};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
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
    pub payload: serde_json::Value,
}

#[graphql_object(context = Context)]
impl Report {
    /// Application which generated the report
    fn application(&self) -> &str {
        &self.application
    }

    /// Output plugin in command service
    fn output_plugin(&self) -> Option<&str> {
        self.output_plugin.as_deref()
    }

    /// Success/Failure
    fn description(&self) -> &str {
        &self.description
    }

    /// Object id
    fn object_id(&self) -> &Uuid {
        &self.object_id
    }

    /// JSON encoded payload
    fn payload(&self) -> FieldResult<String> {
        Ok(serde_json::to_string(&self.payload)?)
    }
}
