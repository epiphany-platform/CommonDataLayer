use std::{collections::HashMap, num::NonZeroU8};

use async_graphql::{FieldResult, InputObject, Json};
use rpc::schema_registry::types::SearchFor;
use serde_json::Value;
use uuid::Uuid;

use crate::types::IntoQueried;

use cdl_dto::materialization::{Filter, Relation};

/// A new view under a schema.
#[derive(Clone, Debug, InputObject)]
pub struct NewView {
    /// The name of the view.
    pub name: String,
    /// The address of the materializer this view caches data in.
    pub materializer_address: String,
    /// Materializer's options encoded in JSON
    pub materializer_options: Json<Value>,
    /// The fields that this view maps with.
    pub fields: Json<HashMap<String, Value>>,
    /// Filters to the fields
    pub filters: Option<Json<Filter>>,
    /// The relations that this view has.
    pub relations: Vec<NewRelation>,
}

/// Relation between a view's schemas
#[derive(Clone, Debug, InputObject, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRelation {
    /// Relation ID stored in Edge Registry
    pub global_id: Uuid,
    /// Unique in view definition
    pub local_id: NonZeroU8,
    /// Looking at relation which direction is important.
    pub search_for: SearchFor,
    /// Subrelations
    pub relations: Vec<NewRelation>,
}

impl IntoQueried for NewRelation {
    type Queried = Relation;

    fn into_queried(self) -> Self::Queried {
        Relation {
            global_id: self.global_id,
            local_id: self.local_id,
            search_for: self.search_for,
            relations: self.relations.into_queried(),
        }
    }
}

impl NewRelation {
    pub fn into_rpc(self) -> rpc::schema_registry::Relation {
        let local_id: u8 = self.local_id.into();

        rpc::schema_registry::Relation {
            global_id: self.global_id.to_string(),
            local_id: local_id as u32,
            search_for: rpc::schema_registry::SearchFor {
                search_for: self.search_for.into(),
            },
            relations: self.relations.into_iter().map(|r| r.into_rpc()).collect(),
        }
    }
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
    pub fields: Option<Json<HashMap<String, Value>>>,
    /// Filters to the fields
    pub filters: Option<Json<Filter>>,
    /// Should filters be updated if not present
    #[graphql(default)]
    pub clean_filters: bool,
    /// The relations that this view has.
    pub relations: Option<Vec<NewRelation>>,
}

impl ViewUpdate {
    pub fn into_rpc(self, id: Uuid) -> FieldResult<rpc::schema_registry::ViewUpdate> {
        let (update_fields, fields) = if let Some(fields) = self.fields {
            (
                true,
                fields
                    .0
                    .into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect(),
            )
        } else {
            (false, Default::default())
        };

        let (update_relations, relations) = if let Some(relations) = self.relations {
            (true, relations.into_iter().map(|r| r.into_rpc()).collect())
        } else {
            (false, Default::default())
        };

        let (update_filters, filters) = match (self.filters, self.clean_filters) {
            (_, true) => (true, Default::default()),
            (filters @ Some(_), _) => (true, filters.map(|f| f.0.into_rpc()).transpose()?),
            _ => (false, Default::default()),
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
            filters,
            update_filters,
            relations,
            update_relations,
        })
    }
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

impl From<OnDemandViewRequest> for rpc::materializer_ondemand::OnDemandRequest {
    fn from(val: OnDemandViewRequest) -> Self {
        let schemas = val
            .schemas
            .into_iter()
            .map(|schema| {
                (
                    schema.id.to_string(),
                    rpc::materializer_ondemand::Schema {
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
