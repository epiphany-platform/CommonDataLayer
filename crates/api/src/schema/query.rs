use std::collections::HashMap;

use async_graphql::{Context, FieldResult, Json, Object};
use itertools::Itertools;
use num_traits::FromPrimitive;
use uuid::Uuid;

use rpc::object_builder::ViewId;
use rpc::schema_registry::Empty;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::schema::context::{EdgeRegistryPool, ObjectBuilderPool, SchemaRegistryPool};
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::{CdlObject, EdgeRelations, SchemaRelation};
use crate::types::schema::{Definition, Schema, SchemaType, View};
use crate::types::view::{MaterializedView, RowDefinition};

#[Object]
/// Schema is the format in which data is to be sent to the Common Data Layer.
impl Schema {
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

    #[graphql(name = "type")]
    async fn schema_type(&self) -> SchemaType {
        self.schema_type
    }

    /// Returns schema definition for given version.
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist,
    /// querying for "=2.1.0" will return "2.1.0" if exist
    #[tracing::instrument(skip(self, context), fields(id))]
    async fn definition(&self, context: &Context<'_>, version: String) -> FieldResult<Definition> {
        let id = self.id.to_string();
        tracing::Span::current().record("id", &id.as_str());
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
        let schema_def = conn
            .get_schema(rpc::schema_registry::VersionedId {
                id,
                version_req: version,
            })
            .await
            .map_err(rpc::error::schema_registry_error)?
            .into_inner();

        Ok(Definition {
            version: schema_def.version,
            definition: Json(serde_json::from_str(&schema_def.definition)?),
        })
    }

    /// All definitions connected to this schema.
    /// Each schema can have only one active definition, under latest version but also contains history for backward compability.
    #[tracing::instrument(skip(self, context), fields(id))]
    async fn definitions(&self, context: &Context<'_>) -> FieldResult<Vec<Definition>> {
        let id = self.id.to_string();
        tracing::Span::current().record("id", &id.as_str());
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
        let rpc_id = rpc::schema_registry::Id { id: id.clone() };

        let versions = conn
            .get_schema_versions(rpc_id)
            .await
            .map_err(rpc::error::schema_registry_error)?
            .into_inner()
            .versions;

        let mut definitions = vec![];
        for version in versions {
            let schema_def = conn
                .get_schema(rpc::schema_registry::VersionedId {
                    id: id.clone(),
                    version_req: format!("={}", version),
                })
                .await
                .map_err(rpc::error::schema_registry_error)?
                .into_inner();

            definitions.push(Definition {
                version: schema_def.version,
                definition: Json(serde_json::from_str(&schema_def.definition)?),
            });
        }

        Ok(definitions)
    }

    /// All views connected to this schema
    #[tracing::instrument(skip(self, context), fields(id))]
    async fn views(&self, context: &Context<'_>) -> FieldResult<Vec<View>> {
        let id = self.id.to_string();
        tracing::Span::current().record("id", &id.as_str());
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
        let rpc_id = rpc::schema_registry::Id { id: id.clone() };

        let views = conn
            .get_all_views_of_schema(rpc_id.clone())
            .await
            .map_err(rpc::error::schema_registry_error)?
            .into_inner()
            .views
            .into_iter()
            .map(|(id, view)| {
                Ok(View {
                    id: id.parse()?,
                    name: view.name,
                    materializer_addr: view.materializer_addr,
                    fields: serde_json::from_str(&view.fields).map_err(Error::ViewFieldError)?,
                })
            })
            .collect::<Result<_>>()?;

        Ok(views)
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Return single schema for given id
    #[tracing::instrument(skip(self, context))]
    async fn schema(&self, context: &Context<'_>, id: Uuid) -> FieldResult<Schema> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
        get_schema(&mut conn, id).await
    }

    /// Return all schemas in database
    #[tracing::instrument(skip(self, context))]
    async fn schemas(&self, context: &Context<'_>) -> FieldResult<Vec<Schema>> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
        let schemas = conn
            .get_all_schemas(Empty {})
            .await
            .map_err(rpc::error::schema_registry_error)?
            .into_inner()
            .schemas
            .into_iter()
            .map(|(schema_id, schema)| {
                Ok(Schema {
                    id: schema_id.parse()?,
                    name: schema.name,
                    insert_destination: schema.insert_destination,
                    query_address: schema.query_address,
                    schema_type: SchemaType::from_i32(schema.schema_type)
                        .ok_or(Error::InvalidSchemaType(schema.schema_type))?,
                })
            })
            .collect::<Result<_>>()?;

        Ok(schemas)
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
        view_id: Uuid,
    ) -> FieldResult<MaterializedView> {
        let mut conn = context.data_unchecked::<ObjectBuilderPool>().get().await?;
        let materialized = conn
            .materialize(utils::tracing::grpc::inject_span(ViewId {
                view_id: view_id.to_string(),
            }))
            .await?
            .into_inner();

        let rows = materialized
            .rows
            .into_iter()
            .map(|row| {
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
            .collect::<Result<_, async_graphql::Error>>()?;

        let options = serde_json::from_str(&materialized.options)?;

        Ok(MaterializedView {
            id: view_id,
            materializer_options: Json(options),
            rows,
        })
    }
}
