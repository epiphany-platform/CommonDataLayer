use std::collections::HashMap;

use async_graphql::{InputObject, Json, SimpleObject};
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

#[derive(Debug, Clone, InputObject)]
pub struct OnDemandViewRequest {
    /// View's UUID
    pub view_id: Uuid,
    /// Schemas with objects. This collection is treated like a hash-map with `schemaId` as a key, therefore `schemaId` should be unique per request.
    pub schemas: Vec<Schema>,
}


#[derive(Debug, Clone, InputObject)]
pub struct Schema {
    /// Schema's UUID
    pub id: Uuid,
    /// List of the object IDs
    pub object_ids: Vec<Uuid>,
}

impl From<OnDemandViewRequest> for rpc::object_builder::View {
    fn from(val: OnDemandViewRequest) -> Self {
        let schemas = val
            .schemas
            .into_iter()
            .map(|schema| {
                (
                    schema.id.to_string(),
                    rpc::object_builder::Schema {
                        object_ids: schema
                            .object_ids
                            .into_iter()
                            .map(|id| id.to_string())
                            .collect(),
                    },
                )
            })
            .collect();
        Self {
            view_id: val.view_id.to_string(),
            schemas,
        }
    }
}
