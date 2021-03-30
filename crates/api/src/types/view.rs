use std::collections::HashMap;

use async_graphql::{FieldResult, InputObject, Json, SimpleObject};
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
    /// The fields that this view maps with.
    pub fields: Json<HashMap<String, String>>,
}

impl View {
    pub fn from_rpc(view: rpc::schema_registry::View) -> FieldResult<Self> {
        Ok(View {
            id: Uuid::parse_str(&view.id)?,
            name: view.name,
            materializer_address: view.materializer_address,
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
    /// The fields that this view maps with.
    pub fields: Json<HashMap<String, String>>,
}

/// An update to a view. Only the provided properties are updated.
#[derive(Debug, InputObject)]
pub struct ViewUpdate {
    /// The ID of the view.
    pub id: Uuid,
    /// The name of the view.
    pub name: Option<String>,
    /// The address of the materializer this view caches data in.
    pub materializer_address: Option<String>,
    /// The fields that this view maps with.
    pub fields: Option<Json<HashMap<String, String>>>,
}
