use std::collections::HashMap;

use async_graphql::{Context, FieldResult, Json, Object};
use itertools::Itertools;
use semver::VersionReq;
use uuid::Uuid;

use crate::error::Result;
use crate::schema::context::{EdgeRegistryPool, OnDemandMaterializerPool, SchemaRegistryPool};
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::{CdlObject, EdgeRelations, SchemaRelation};
use crate::types::schema::{Definition, FullSchema};
use crate::types::view::View;
use crate::types::view::{MaterializedView, RowDefinition};
use crate::{config::Config, types::view::OnDemandViewRequest};
use rpc::schema_registry::types::SchemaType;

#[Object]
/// Schema is the format in which data is to be sent to the Common Data Layer.
impl FullSchema {
    /// Random UUID assigned on creation
    async fn id(&self) -> &Uuid {
        &self.id
    }

    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    async fn name(&self) -> &str {
        &self.name
    }

    /// Message queue insert_destination to which data is inserted by data-router.
    async fn insert_destination(&self) -> &str {
        &self.insert_destination
    }

    /// Address of the query service responsible for retrieving data from DB
    async fn query_address(&self) -> &str {
        &self.query_address
    }

    /// Whether this schema represents documents or timeseries data.
    #[graphql(name = "type")]
    async fn schema_type(&self) -> SchemaType {
        self.schema_type
    }

    /// Returns schema definition for given version.
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist,
    /// querying for "=2.1.0" will return "2.1.0" if exist
    #[tracing::instrument(skip(self))]
    async fn definition(&self, version_req: String) -> FieldResult<&Definition> {
        let version_req = VersionReq::parse(&version_req)?;
        let definition = self
            .get_definition(version_req)
            .ok_or("No definition matches the given requirement")?;

        Ok(definition)
    }

    /// All definitions connected to this schema.
    /// Each schema can have only one active definition, under latest version but also contains history for backward compability.
    async fn definitions(&self) -> &Vec<Definition> {
        &self.definitions
    }

    /// All views belonging to this schema.
    async fn views(&self) -> &Vec<View> {
        &self.views
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Return single schema for given id
    #[tracing::instrument(skip(self, context))]
    async fn schema(&self, context: &Context<'_>, id: Uuid) -> FieldResult<FullSchema> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        get_schema(&mut conn, id).await
    }

    /// Return all schemas in database
    #[tracing::instrument(skip(self, context))]
    async fn schemas(&self, context: &Context<'_>) -> FieldResult<Vec<FullSchema>> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        let schemas = conn
            .get_all_full_schemas(rpc::schema_registry::Empty {})
            .await?
            .into_inner()
            .schemas;

        schemas.into_iter().map(FullSchema::from_rpc).collect()
    }

    /// Return single view for given id
    #[tracing::instrument(skip(self, context))]
    async fn view(&self, context: &Context<'_>, id: Uuid) -> FieldResult<View> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
        get_view(&mut conn, id).await
    }

    /// Return a single object from the query router
    #[tracing::instrument(skip(self, context))]
    async fn object(
        &self,
        context: &Context<'_>,
        object_id: Uuid,
        schema_id: Uuid,
    ) -> FieldResult<CdlObject> {
        let client = reqwest::Client::new();

        let bytes = client
            .post(&format!(
                "{}/single/{}",
                &context.data_unchecked::<Config>().query_router_addr,
                object_id
            ))
            .header("SCHEMA_ID", schema_id.to_string())
            .body("{}")
            .send()
            .await?
            .bytes()
            .await?;

        Ok(CdlObject {
            object_id,
            data: serde_json::from_slice(&bytes[..])?,
        })
    }

    /// Return a map of objects selected by ID from the query router
    #[tracing::instrument(skip(self, context))]
    async fn objects(
        &self,
        context: &Context<'_>,
        object_ids: Vec<Uuid>,
        schema_id: Uuid,
    ) -> FieldResult<Vec<CdlObject>> {
        let client = reqwest::Client::new();

        let id_list = object_ids.iter().join(",");

        let values: HashMap<Uuid, serde_json::Value> = client
            .get(&format!(
                "{}/multiple/{}",
                &context.data_unchecked::<Config>().query_router_addr,
                id_list
            ))
            .header("SCHEMA_ID", schema_id.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(values
            .into_iter()
            .map(|(object_id, data)| CdlObject {
                object_id,
                data: Json(data),
            })
            .collect::<Vec<CdlObject>>())
    }

    /// Return a map of all objects (keyed by ID) in a schema from the query router
    #[tracing::instrument(skip(self, context))]
    async fn schema_objects(
        &self,
        context: &Context<'_>,
        schema_id: Uuid,
    ) -> FieldResult<Vec<CdlObject>> {
        let client = reqwest::Client::new();

        let values: HashMap<Uuid, serde_json::Value> = client
            .get(&format!(
                "{}/schema",
                &context.data_unchecked::<Config>().query_router_addr,
            ))
            .header("SCHEMA_ID", schema_id.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(values
            .into_iter()
            .map(|(object_id, data)| CdlObject {
                object_id,
                data: Json(data),
            })
            .collect::<Vec<CdlObject>>())
    }

    /// Return schema `parent` is in `relation_id` relation with
    #[tracing::instrument(skip(self, context))]
    async fn relation(
        &self,
        context: &Context<'_>,
        relation_id: Uuid,
        parent_schema_id: Uuid,
    ) -> FieldResult<Option<Uuid>> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        Ok(conn
            .get_relation(rpc::edge_registry::RelationQuery {
                relation_id: relation_id.to_string(),
                parent_schema_id: parent_schema_id.to_string(),
            })
            .await?
            .into_inner()
            .child_schema_id
            .map(|s| s.parse())
            .transpose()?)
    }

    /// Return all relations `parent` is in
    #[tracing::instrument(skip(self, context))]
    async fn schema_relations(
        &self,
        context: &Context<'_>,
        parent_schema_id: Uuid,
    ) -> FieldResult<Vec<SchemaRelation>> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        conn.get_schema_relations(rpc::edge_registry::SchemaId {
            schema_id: parent_schema_id.to_string(),
        })
        .await?
        .into_inner()
        .items
        .into_iter()
        .map(|entry| {
            Ok(SchemaRelation {
                parent_schema_id,
                relation_id: entry.relation_id.parse()?,
                child_schema_id: entry.child_schema_id.parse()?,
            })
        })
        .collect::<Result<Vec<_>, async_graphql::Error>>()
    }

    /// List all relations between schemas stored in database
    #[tracing::instrument(skip(self, context))]
    async fn all_relations(&self, context: &Context<'_>) -> FieldResult<Vec<SchemaRelation>> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        conn.list_relations(rpc::edge_registry::Empty {})
            .await?
            .into_inner()
            .items
            .into_iter()
            .map(|entry| {
                Ok(SchemaRelation {
                    relation_id: entry.relation_id.parse()?,
                    parent_schema_id: entry.parent_schema_id.parse()?,
                    child_schema_id: entry.child_schema_id.parse()?,
                })
            })
            .collect::<Result<Vec<_>, async_graphql::Error>>()
    }

    /// Return all objects that `parent` object is in `relation_id` relation with
    #[tracing::instrument(skip(self, context))]
    async fn edge(
        &self,
        context: &Context<'_>,
        relation_id: Uuid,
        parent_object_id: Uuid,
    ) -> FieldResult<Vec<Uuid>> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        conn.get_edge(rpc::edge_registry::RelationIdQuery {
            relation_id: relation_id.to_string(),
            parent_object_id: parent_object_id.to_string(),
        })
        .await?
        .into_inner()
        .child_object_ids
        .into_iter()
        .map(|entry| Ok(entry.parse()?))
        .collect::<Result<Vec<_>, async_graphql::Error>>()
    }

    /// Return all relations that `parent` object is in
    #[tracing::instrument(skip(self, context))]
    async fn edges(
        &self,
        context: &Context<'_>,
        parent_object_id: Uuid,
    ) -> FieldResult<Vec<EdgeRelations>> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        conn.get_edges(rpc::edge_registry::ObjectIdQuery {
            object_id: parent_object_id.to_string(),
        })
        .await?
        .into_inner()
        .relations
        .into_iter()
        .map(|entry| {
            Ok(EdgeRelations {
                relation_id: entry.relation_id.parse()?,
                parent_object_id: entry.parent_object_id.parse()?,
                child_object_ids: entry
                    .child_object_ids
                    .into_iter()
                    .map(|s| Ok(s.parse()?))
                    .collect::<Result<Vec<_>, async_graphql::Error>>()?,
            })
        })
        .collect::<Result<Vec<_>, async_graphql::Error>>()
    }

    /// On demand materialized view
    #[tracing::instrument(skip(self, context))]
    async fn on_demand_view(
        &self,
        context: &Context<'_>,
        request: OnDemandViewRequest,
    ) -> FieldResult<MaterializedView> {
        use futures::TryStreamExt;

        let mut conn = context
            .data_unchecked::<OnDemandMaterializerPool>()
            .get()
            .await?;
        let view_id = request.view_id;
        let materialized = conn
            .materialize(utils::tracing::grpc::inject_span(request.into()))
            .await?
            .into_inner();

        let rows = materialized
            .map_err(async_graphql::Error::from)
            .and_then(|row| async move {
                let fields = row
                    .fields
                    .into_iter()
                    .map(|(field_name, field)| {
                        let field = serde_json::from_str(&field)?;
                        Ok((field_name, Json(field)))
                    })
                    .collect::<Result<_, async_graphql::Error>>()?;

                Ok(RowDefinition {
                    object_id: row.object_id.parse()?,
                    fields,
                })
            })
            .try_collect::<_>()
            .await?; //In the future we could probably use graphQL @stream directive when its standarized.

        Ok(MaterializedView { id: view_id, rows })
    }
}
