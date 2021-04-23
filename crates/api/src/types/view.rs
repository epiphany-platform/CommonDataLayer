use std::collections::HashMap;

use async_graphql::{FieldResult, InputObject, Json, SimpleObject};
use serde_json::Value;
use uuid::Uuid;

/// A view under a schema.
#[derive(Debug, SimpleObject)]
pub struct View {
    /// The ID of the view.
    pub id: Uuid,
    /// The name of the view.
    pub name: String,
    /// The address of the materializer this view caches data in.
    pub materializer_address: String,
    /// Materializer's options encoded in JSON
    pub materializer_options: Json<Value>,
    /// The fields that this view maps with.
    pub fields: Json<HashMap<String, String>>,
}

impl View {
    pub fn from_rpc(view: rpc::schema_registry::View) -> FieldResult<Self> {
        Ok(View {
            id: Uuid::parse_str(&view.id)?,
            name: view.name,
            materializer_address: view.materializer_address,
            materializer_options: serde_json::from_str(&view.materializer_options)?,
            fields: Json(view.fields),
        })
    }
}

/// A new view under a schema.
#[derive(Clone, Debug, InputObject)]
pub struct NewView {
    /// The ID of the schema this view will belong to.
    pub schema_id: Uuid,
    /// The name of the view.
    pub name: String,
    /// The address of the materializer this view caches data in.
    pub materializer_address: String,
    /// Materializer's options encoded in JSON
    pub materializer_options: Json<Value>,
    /// The fields that this view maps with.
    pub fields: Json<HashMap<String, String>>,
}

/// An update to a view. Only the provided properties are updated.
#[derive(Debug, InputObject)]
pub struct ViewUpdate {
    /// The name of the view.
    pub name: Option<String>,
    /// The address of the materializer this view caches data in.
    pub materializer_address: Option<String>,
    /// Materializer's options encoded in JSON
    pub materializer_options: Option<Json<Value>>,
    /// The fields that this view maps with.
    pub fields: Option<Json<HashMap<String, String>>>,
}

impl ViewUpdate {
    pub fn into_rpc(self, id: Uuid) -> FieldResult<rpc::schema_registry::ViewUpdate> {
        let (update_fields, fields) = if let Some(fields) = self.fields {
            (true, fields.0)
        } else {
            (false, HashMap::default())
        };

        Ok(rpc::schema_registry::ViewUpdate {
            id: id.to_string(),
            name: self.name.clone(),
            materializer_address: self.materializer_address.clone(),
            materializer_options: self
                .materializer_options
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?
                .unwrap_or_default(),
            fields,
            update_fields,
        })
    }
}

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
