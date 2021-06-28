use std::collections::HashMap;

use async_graphql::{FieldResult, Json, SimpleObject};
use serde_json::Value;
use uuid::Uuid;

use cdl_dto::materialization::{Filter, Relation};
use cdl_dto::TryFromRpc;

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
    pub fields: Json<HashMap<String, Value>>,
    /// The relations that this view has.
    pub relations: Vec<Relation>,
    /// Filters used to narrow source objects.
    pub filters: Option<Filter>,
}

impl View {
    pub fn from_rpc(view: rpc::schema_registry::View) -> FieldResult<Self> {
        Ok(Self {
            id: Uuid::parse_str(&view.id)?,
            name: view.name,
            materializer_address: view.materializer_address,
            materializer_options: serde_json::from_str(&view.materializer_options)?,
            fields: Json(
                view.fields
                    .into_iter()
                    .map(|(k, v)| Ok((k, serde_json::from_str(&v)?)))
                    .collect::<FieldResult<_>>()?,
            ),
            filters: view.filters.map(TryFromRpc::try_from_rpc).transpose()?,
            relations: view
                .relations
                .into_iter()
                .map(TryFromRpc::try_from_rpc)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// A view under a schema.
#[derive(Debug, SimpleObject)]
pub struct FullView {
    /// The ID of the view.
    pub id: Uuid,
    /// The ID of the base schema.
    pub base_schema_id: Uuid,
    /// The name of the view.
    pub name: String,
    /// The address of the materializer this view caches data in.
    pub materializer_address: String,
    /// Materializer's options encoded in JSON
    pub materializer_options: Json<Value>,
    /// The fields that this view maps with.
    pub fields: Json<HashMap<String, Value>>,
    /// The relations that this view has.
    pub relations: Vec<Relation>,
}

impl FullView {
    pub fn from_rpc(view: rpc::schema_registry::FullView) -> FieldResult<Self> {
        Ok(Self {
            id: Uuid::parse_str(&view.id)?,
            base_schema_id: Uuid::parse_str(&view.base_schema_id)?,
            name: view.name,
            materializer_address: view.materializer_address,
            materializer_options: serde_json::from_str(&view.materializer_options)?,
            fields: Json(
                view.fields
                    .into_iter()
                    .map(|(k, v)| Ok((k, serde_json::from_str(&v)?)))
                    .collect::<FieldResult<_>>()?,
            ),
            relations: view
                .relations
                .into_iter()
                .map(TryFromRpc::try_from_rpc)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, SimpleObject, serde::Deserialize)]
pub struct MaterializedView {
    /// Source view's UUID
    pub id: Uuid,
    /// Materialized objects
    pub rows: Vec<RowDefinition>,
}

#[derive(Debug, SimpleObject, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowDefinition {
    /// Object UUIDs
    pub object_ids: Vec<Uuid>,
    /// Materialized fields
    pub fields: HashMap<String, Json<Value>>,
}
