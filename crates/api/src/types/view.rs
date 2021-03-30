use std::collections::HashMap;

use juniper::FieldResult;
use uuid::Uuid;

/// A view under a schema.
#[derive(Debug, juniper::GraphQLObject)]
pub struct View {
    /// The ID of the view.
    pub id: Uuid,
    /// The name of the view.
    pub name: String,
    /// The address of the materializer this view caches data in.
    pub materializer_address: String,
    /// The fields that this view maps with.
    pub fields: String,
}

impl View {
    pub fn from_rpc(view: rpc::schema_registry::View) -> FieldResult<Self> {
        Ok(View {
            id: Uuid::parse_str(&view.id)?,
            name: view.name,
            materializer_address: view.materializer_address,
            fields: serde_json::to_string(
                &view
                    .fields
                    .into_iter()
                    .map(|(key, value)| Ok((key, serde_json::from_str(&value)?)))
                    .collect::<FieldResult<HashMap<_, _>>>()?,
            )?,
        })
    }
}

/// A new view under a schema.
#[derive(Clone, Debug, juniper::GraphQLInputObject)]
pub struct NewView {
    /// The ID of the schema this view will belong to.
    pub schema_id: Uuid,
    /// The name of the view.
    pub name: String,
    /// The address of the materializer this view caches data in.
    pub materializer_address: String,
    /// The fields that this view maps with.
    pub fields: String,
}

/// An update to a view. Only the provided properties are updated.
#[derive(Debug, juniper::GraphQLInputObject)]
pub struct ViewUpdate {
    /// The ID of the view.
    pub id: Uuid,
    /// The name of the view.
    pub name: Option<String>,
    /// The address of the materializer this view caches data in.
    pub materializer_address: Option<String>,
    /// The fields that this view maps with.
    pub fields: Option<String>,
}
