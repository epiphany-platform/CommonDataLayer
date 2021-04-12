use std::collections::HashMap;

use async_graphql::{Json, SimpleObject};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, SimpleObject)]
pub struct MaterializedView {
    /// Source view's UUID
    pub id: Uuid,
    /// Materializer-specific options, available only for debugging purposes.
    pub materializer_options: Json<Value>,
    /// Materialized objects
    pub rows: Vec<RowDefinition>,
}

#[derive(Debug, SimpleObject)]
pub struct RowDefinition {
    /// Object's UUID
    pub object_id: Uuid,
    /// Materialized fields
    pub fields: HashMap<String, Json<Value>>,
}
