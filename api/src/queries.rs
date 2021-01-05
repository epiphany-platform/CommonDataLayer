use crate::error::Result;
use crate::schema::*;
use crate::{
    context::{Context, SchemaRegistryConn},
    error::Error,
};
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};
use num_traits::{FromPrimitive, ToPrimitive};
use rpc::schema_registry::Empty;
use uuid::Uuid;

pub type GQLSchema = RootNode<'static, Query, QueryMut, EmptySubscription<Context>>;

pub fn schema() -> GQLSchema {
    GQLSchema::new(Query, QueryMut, EmptySubscription::new())
}

pub struct QueryMut;

#[graphql_object(context = Context)]
impl QueryMut {
    async fn add_schema(context: &Context, new: NewSchema) -> FieldResult<Schema> {
        log::debug!("add schema {:?}", new);
        let mut conn = context.connect_to_registry().await?;

        let NewSchema {
            name,
            query_address,
            topic,
            definition,
            schema_type,
        } = new;

        let rpc_schema_type: i32 = schema_type.to_i32().unwrap(); // Unwrap because we for sure can build i32 from enum

        let id = conn
            .add_schema(rpc::schema_registry::NewSchema {
                id: "".into(),
                name: name.clone(),
                query_address: query_address.clone(),
                topic: topic.clone(),
                definition,
                schema_type: rpc_schema_type,
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .id
            .parse()?;

        Ok(Schema {
            id,
            name,
            query_address,
            topic,
            schema_type,
        })
    }

    async fn add_schema_definition(
        context: &Context,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        log::debug!(
            "add schema definition for {:?} - {:?}",
            schema_id,
            new_version
        );
        let mut conn = context.connect_to_registry().await?;

        let NewVersion {
            definition,
            version,
        } = new_version;

        conn.add_schema_version(rpc::schema_registry::NewSchemaVersion {
            id: schema_id.to_string(),
            version: version.clone(),
            definition: definition.clone(),
        })
        .await
        .map_err(rpc::error::registry_error)?;

        Ok(Definition {
            definition,
            version,
        })
    }

    async fn add_view(context: &Context, schema_id: Uuid, new_view: NewView) -> FieldResult<View> {
        log::debug!("add view for {} - {:?}", schema_id, new_view);

        let NewView { name, expression } = new_view.clone();
        let mut conn = context.connect_to_registry().await?;
        let id = conn
            .add_view_to_schema(rpc::schema_registry::NewSchemaView {
                schema_id: schema_id.to_string(),
                view_id: "".into(),
                name,
                jmespath: expression,
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner()
            .id;

        Ok(View {
            id: id.parse()?,
            name: new_view.name,
            expression: new_view.expression,
        })
    }

    async fn update_view(context: &Context, id: Uuid, update: UpdateView) -> FieldResult<View> {
        log::debug!("update view for {} - {:?}", id, update);

        let mut conn = context.connect_to_registry().await?;

        let UpdateView { name, expression } = update;

        conn.update_view(rpc::schema_registry::UpdatedView {
            id: id.to_string(),
            name: name.clone(),
            jmespath: expression.clone(),
        })
        .await
        .map_err(rpc::error::registry_error)?;

        get_view(&mut conn, id).await
    }

    async fn update_schema(
        context: &Context,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<Schema> {
        log::debug!("update schema for {} - {:?}", id, update);

        let mut conn = context.connect_to_registry().await?;

        let UpdateSchema {
            name,
            query_address: address,
            topic,
            schema_type,
        } = update;

        conn.update_schema_metadata(rpc::schema_registry::SchemaMetadataUpdate {
            id: id.to_string(),
            name,
            address,
            topic,
            schema_type: schema_type.and_then(|s| s.to_i32()),
        })
        .await
        .map_err(rpc::error::registry_error)?;

        get_schema(&mut conn, id).await
    }
}

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

    /// Kafka topic to which data is inserted by data-router.
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
        let id = self.id.to_string();
        let mut conn = context.connect_to_registry().await?;
        let schema_def = conn
            .get_schema(rpc::schema_registry::VersionedId {
                id,
                version_req: version.clone(),
            })
            .await
            .map_err(rpc::error::registry_error)?
            .into_inner();

        Ok(Definition {
            version: schema_def.version,
            definition: schema_def.definition,
        })
    }

    /// All definitions connected to this schema.
    /// Each schema can have only one active definition, under latest version but also contains history for backward compability.
    async fn definitions(&self, context: &Context) -> FieldResult<Vec<Definition>> {
        let mut conn = context.connect_to_registry().await?;
        let id = self.id.to_string();
        let rpc_id = rpc::schema_registry::Id { id: id.clone() };

        let versions = conn
            .get_schema_versions(rpc_id.clone())
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

    /// All views connected to this schema
    async fn views(&self, context: &Context) -> FieldResult<Vec<View>> {
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
                    expression: view.jmespath,
                })
            })
            .collect::<Result<_>>()?;

        Ok(views)
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// Return single schema for given id
    async fn schema(context: &Context, id: Uuid) -> FieldResult<Schema> {
        let mut conn = context.connect_to_registry().await?;
        get_schema(&mut conn, id).await
    }

    /// Return all schemas in database
    async fn schemas(context: &Context) -> FieldResult<Vec<Schema>> {
        log::debug!("get all schemas");
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

    /// Return single view for given id
    async fn view(context: &Context, id: Uuid) -> FieldResult<View> {
        let mut conn = context.connect_to_registry().await?;
        get_view(&mut conn, id).await
    }
}

async fn get_view(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<View> {
    log::debug!("get view: {:?}", id);
    let view = conn
        .get_view(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    Ok(View {
        id,
        name: view.name,
        expression: view.jmespath,
    })
}

async fn get_schema(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<Schema> {
    log::debug!("get schema: {:?}", id);
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(rpc::error::registry_error)?
        .into_inner();

    let schema = Schema {
        id,
        name: schema.name,
        topic: schema.topic,
        query_address: schema.query_address,
        schema_type: SchemaType::from_i32(schema.schema_type)
            .ok_or(Error::InvalidSchemaType(schema.schema_type))?,
    };

    log::debug!("schema: {:?}", schema);

    Ok(schema)
}
