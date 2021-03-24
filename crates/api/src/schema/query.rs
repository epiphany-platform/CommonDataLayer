use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::schema::context::Context;
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::CdlObject;
use crate::types::schema::*;

use itertools::Itertools;
use juniper::{graphql_object, FieldResult};
use num_traits::FromPrimitive;
use rpc::schema_registry::Empty;
use tracing::Instrument;
use uuid::Uuid;

#[graphql_object(context = Context)]
/// Schema is the format in which data is to be sent to the Common Data Layer.
impl Schema {
    /// Random UUID assigned on creation
    fn id(&self) -> &Uuid {
        &self.id
    }

    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    fn name(&self) -> &str {
        &self.name
    }

    /// Message queue topic to which data is inserted by data-router.
    fn topic(&self) -> &str {
        &self.topic
    }

    /// Address of the query service responsible for retrieving data from DB
    fn query_address(&self) -> &str {
        &self.query_address
    }

    #[graphql(name = "type")]
    fn schema_type(&self) -> SchemaType {
        self.schema_type
    }

    /// Returns schema definition for given version.
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist,
    /// querying for "=2.1.0" will return "2.1.0" if exist
    async fn definition(&self, context: &Context, version: String) -> FieldResult<Definition> {
        let span = tracing::trace_span!("query_definition", ?self.id, ?version);
        async move {
            let id = self.id.to_string();
            let mut conn = context.connect_to_registry().await?;
            let schema_def = conn
                .get_schema(rpc::schema_registry::VersionedId {
                    id,
                    version_req: version,
                })
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner();

            Ok(Definition {
                version: schema_def.version,
                definition: schema_def.definition,
            })
        }
        .instrument(span)
        .await
    }

    /// All definitions connected to this schema.
    /// Each schema can have only one active definition, under latest version but also contains history for backward compability.
    async fn definitions(&self, context: &Context) -> FieldResult<Vec<Definition>> {
        let span = tracing::trace_span!("query_definitions", ?self.id);
        async move {
            let mut conn = context.connect_to_registry().await?;
            let id = self.id.to_string();
            let rpc_id = rpc::schema_registry::Id { id: id.clone() };

            let versions = conn
                .get_schema_versions(rpc_id)
                .await
                .map_err(rpc::error::registry_error)?
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
                    .map_err(rpc::error::registry_error)?
                    .into_inner();

                definitions.push(Definition {
                    version: schema_def.version,
                    definition: schema_def.definition,
                });
            }

            Ok(definitions)
        }
        .instrument(span)
        .await
    }

    /// All views connected to this schema
    async fn views(&self, context: &Context) -> FieldResult<Vec<View>> {
        let span = tracing::trace_span!("query_views", ?self.id);
        async move {
            let mut conn = context.connect_to_registry().await?;
            let id = self.id.to_string();
            let rpc_id = rpc::schema_registry::Id { id: id.clone() };

            let views = conn
                .get_all_views_of_schema(rpc_id.clone())
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner()
                .views
                .into_iter()
                .map(|(id, view)| {
                    Ok(View {
                        id: id.parse()?,
                        name: view.name,
                        materializer_addr: view.materializer_addr,
                        fields: view.fields,
                    })
                })
                .collect::<Result<_>>()?;

            Ok(views)
        }
        .instrument(span)
        .await
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// Return single schema for given id
    async fn schema(context: &Context, id: Uuid) -> FieldResult<Schema> {
        let span = tracing::trace_span!("query_schema", ?id);
        async move {
            let mut conn = context.connect_to_registry().await?;
            get_schema(&mut conn, id).await
        }
        .instrument(span)
        .await
    }

    /// Return all schemas in database
    async fn schemas(context: &Context) -> FieldResult<Vec<Schema>> {
        let span = tracing::trace_span!("query_schemas");
        async move {
            let mut conn = context.connect_to_registry().await?;
            let schemas = conn
                .get_all_schemas(Empty {})
                .await
                .map_err(rpc::error::registry_error)?
                .into_inner()
                .schemas
                .into_iter()
                .map(|(schema_id, schema)| {
                    Ok(Schema {
                        id: schema_id.parse()?,
                        name: schema.name,
                        topic: schema.topic,
                        query_address: schema.query_address,
                        schema_type: SchemaType::from_i32(schema.schema_type)
                            .ok_or(Error::InvalidSchemaType(schema.schema_type))?,
                    })
                })
                .collect::<Result<_>>()?;

            Ok(schemas)
        }
        .instrument(span)
        .await
    }

    /// Return single view for given id
    async fn view(context: &Context, id: Uuid) -> FieldResult<View> {
        let span = tracing::trace_span!("query_view", ?id);
        async move {
            let mut conn = context.connect_to_registry().await?;
            get_view(&mut conn, id).await
        }
        .instrument(span)
        .await
    }

    /// Return a single object from the query router
    async fn object(object_id: Uuid, schema_id: Uuid, context: &Context) -> FieldResult<CdlObject> {
        let span = tracing::trace_span!("query_object", ?object_id, ?schema_id);
        async move {
            let client = reqwest::Client::new();

            let bytes = client
                .post(&format!(
                    "{}/single/{}",
                    &context.config().query_router_addr,
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
        object_ids: Vec<Uuid>,
        schema_id: Uuid,
        context: &Context,
    ) -> FieldResult<Vec<CdlObject>> {
        let span = tracing::trace_span!("query_objects", ?object_ids, ?schema_id);
        async move {
            let client = reqwest::Client::new();

            let id_list = object_ids.iter().join(",");

            let values: HashMap<Uuid, serde_json::Value> = client
                .get(&format!(
                    "{}/multiple/{}",
                    &context.config().query_router_addr,
                    id_list
                ))
                .header("SCHEMA_ID", schema_id.to_string())
                .send()
                .await?
                .json()
                .await?;

            Ok(values
                .into_iter()
                .map(|(object_id, data)| CdlObject { object_id, data })
                .collect::<Vec<CdlObject>>())
        }
        .instrument(span)
        .await
    }

    /// Return a map of all objects (keyed by ID) in a schema from the query router
    async fn schema_objects(schema_id: Uuid, context: &Context) -> FieldResult<Vec<CdlObject>> {
        let span = tracing::trace_span!("query_schema_objects", ?schema_id);
        async move {
            let client = reqwest::Client::new();

            let values: HashMap<Uuid, serde_json::Value> = client
                .get(&format!("{}/schema", &context.config().query_router_addr,))
                .header("SCHEMA_ID", schema_id.to_string())
                .send()
                .await?
                .json()
                .await?;

            Ok(values
                .into_iter()
                .map(|(object_id, data)| CdlObject { object_id, data })
                .collect::<Vec<CdlObject>>())
        }
        .instrument(span)
        .await
    }
}
