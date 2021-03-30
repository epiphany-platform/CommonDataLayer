use std::collections::HashMap;

use async_graphql::{Context, FieldResult, Json, Object};
use itertools::Itertools;
use semver::VersionReq;
use tracing::Instrument;
use uuid::Uuid;

use crate::config::Config;
use crate::schema::context::SchemaRegistryPool;
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::CdlObject;
use crate::types::schema::{Definition, FullSchema};
use crate::types::view::View;
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

    /// Message queue topic to which data is inserted by data-router.
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
    async fn schema(&self, context: &Context<'_>, id: Uuid) -> FieldResult<FullSchema> {
        let span = tracing::trace_span!("query_schema", ?id);

        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
            get_schema(&mut conn, id).await
        }
        .instrument(span)
        .await
    }

    /// Return all schemas in database
    async fn schemas(&self, context: &Context<'_>) -> FieldResult<Vec<FullSchema>> {
        let span = tracing::trace_span!("query_schemas");
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

            let schemas = conn
                .get_all_full_schemas(rpc::schema_registry::Empty {})
                .await?
                .into_inner()
                .schemas;

            schemas.into_iter().map(FullSchema::from_rpc).collect()
        }
        .instrument(span)
        .await
    }

    /// Return single view for given id
    async fn view(&self, context: &Context<'_>, id: Uuid) -> FieldResult<View> {
        let span = tracing::trace_span!("query_view", ?id);
        async move {
            let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;
            get_view(&mut conn, id).await
        }
        .instrument(span)
        .await
    }

    /// Return a single object from the query router
    async fn object(
        &self,
        context: &Context<'_>,
        object_id: Uuid,
        schema_id: Uuid,
    ) -> FieldResult<CdlObject> {
        let span = tracing::trace_span!("query_object", ?object_id, ?schema_id);
        async move {
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
        .instrument(span)
        .await
    }

    /// Return a map of objects selected by ID from the query router
    async fn objects(
        &self,
        context: &Context<'_>,
        object_ids: Vec<Uuid>,
        schema_id: Uuid,
    ) -> FieldResult<Vec<CdlObject>> {
        let span = tracing::trace_span!("query_objects", ?object_ids, ?schema_id);
        async move {
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
        .instrument(span)
        .await
    }

    /// Return a map of all objects (keyed by ID) in a schema from the query router
    async fn schema_objects(
        &self,
        context: &Context<'_>,
        schema_id: Uuid,
    ) -> FieldResult<Vec<CdlObject>> {
        let span = tracing::trace_span!("query_schema_objects", ?schema_id);
        async move {
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
        .instrument(span)
        .await
    }
}
