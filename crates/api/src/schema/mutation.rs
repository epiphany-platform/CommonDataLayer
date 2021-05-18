use crate::schema::utils::{get_schema, get_view};
use crate::types::data::{InputMessage, ObjectRelations};
use crate::types::schema::{Definition, FullSchema, NewSchema, NewVersion, UpdateSchema};
use crate::types::view::{NewView, View, ViewUpdate};
use crate::{error::Error, settings::Settings};
use crate::{
    schema::context::{EdgeRegistryPool, SchemaRegistryPool},
    types::view::FullView,
    types::IntoQueried,
};
use async_graphql::{Context, FieldResult, Object};
use cdl_dto::ingestion::OwnedInsertMessage;
use serde_json::value::to_raw_value;
use utils::current_timestamp;
use uuid::Uuid;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[tracing::instrument(skip(self, context))]
    async fn add_schema(&self, context: &Context<'_>, new: NewSchema) -> FieldResult<FullSchema> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        let new_id = conn.add_schema(new.into_rpc()?).await?.into_inner().id;
        let schema = conn
            .get_full_schema(rpc::schema_registry::Id { id: new_id })
            .await?
            .into_inner();

        FullSchema::from_rpc(schema)
    }

    #[tracing::instrument(skip(self, context))]
    async fn add_schema_definition(
        &self,
        context: &Context<'_>,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        conn.add_schema_version(rpc::schema_registry::NewSchemaVersion {
            id: schema_id.to_string(),
            definition: rpc::schema_registry::SchemaDefinition {
                version: new_version.version.clone(),
                definition: serde_json::to_vec(&new_version.definition)?,
            },
        })
        .await?;

        Ok(Definition {
            definition: new_version.definition,
            version: new_version.version,
        })
    }

    #[tracing::instrument(skip(self, context))]
    async fn add_view(
        &self,
        context: &Context<'_>,
        schema_id: Uuid,
        new_view: NewView,
    ) -> FieldResult<View> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        let id = conn
            .add_view_to_schema(rpc::schema_registry::NewView {
                base_schema_id: schema_id.to_string(),
                name: new_view.name.clone(),
                materializer_address: new_view.materializer_address.clone(),
                materializer_options: serde_json::to_string(&new_view.materializer_options)?,
                fields: new_view
                    .fields
                    .0
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
                filters: new_view
                    .filters
                    .clone()
                    .map(|f| f.0.into_rpc())
                    .transpose()?,
                relations: new_view
                    .relations
                    .iter()
                    .cloned()
                    .map(|r| r.into_rpc())
                    .collect(),
            })
            .await
            .map_err(|source| rpc::error::ClientError::QueryError { source })?
            .into_inner()
            .id;

        Ok(View {
            id: id.parse()?,
            name: new_view.name,
            materializer_address: new_view.materializer_address,
            materializer_options: new_view.materializer_options,
            fields: new_view.fields,
            filters: new_view.filters.map(|f| f.0),
            relations: new_view.relations.into_queried(),
        })
    }

    #[tracing::instrument(skip(self, context))]
    async fn update_view(
        &self,
        context: &Context<'_>,
        id: Uuid,
        update: ViewUpdate,
    ) -> FieldResult<FullView> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        conn.update_view(update.into_rpc(id)?)
            .await
            .map_err(|source| rpc::error::ClientError::QueryError { source })?;

        get_view(&mut conn, id).await
    }

    #[tracing::instrument(skip(self, context))]
    async fn update_schema(
        &self,
        context: &Context<'_>,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<FullSchema> {
        let mut conn = context.data_unchecked::<SchemaRegistryPool>().get().await?;

        conn.update_schema(update.into_rpc(id))
            .await
            .map_err(|source| rpc::error::ClientError::QueryError { source })?;
        get_schema(&mut conn, id).await
    }

    #[tracing::instrument(skip(self, context))]
    async fn insert_message(
        &self,
        context: &Context<'_>,
        message: InputMessage,
    ) -> FieldResult<bool> {
        let publisher = context.data_unchecked::<Settings>().publisher().await?;
        let payload = serde_json::to_vec(&OwnedInsertMessage {
            object_id: message.object_id,
            schema_id: message.schema_id,
            data: to_raw_value(&message.payload.0).unwrap(), // serde_json::Value -> RawValue should never fail
            timestamp: current_timestamp(),
        })?;

        publisher
            .publish_message(
                &context.data_unchecked::<Settings>().insert_destination,
                "",
                payload,
            )
            .await
            .map_err(Error::PublisherError)?;
        Ok(true)
    }

    #[tracing::instrument(skip(self, context))]
    async fn insert_batch(
        &self,
        context: &Context<'_>,
        messages: Vec<InputMessage>,
    ) -> FieldResult<bool> {
        let publisher = context.data_unchecked::<Settings>().publisher().await?;
        let order_group_id = Uuid::new_v4().to_string();

        for message in messages {
            let payload = serde_json::to_vec(&OwnedInsertMessage {
                object_id: message.object_id,
                schema_id: message.schema_id,
                data: to_raw_value(&message.payload.0).unwrap(), // serde_json::Value -> RawValue should never fail
                timestamp: current_timestamp(),
            })?;

            publisher
                .publish_message(
                    &context.data_unchecked::<Settings>().insert_destination,
                    &order_group_id,
                    payload,
                )
                .await
                .map_err(Error::PublisherError)?;
        }
        Ok(true)
    }

    #[tracing::instrument(skip(self, context))]
    /// Add new relation, return generated `relation_id`
    async fn add_relation(
        &self,
        context: &Context<'_>,
        parent_schema_id: Uuid,
        child_schema_id: Uuid,
    ) -> FieldResult<Uuid> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        Ok(conn
            .add_relation(rpc::edge_registry::SchemaRelation {
                parent_schema_id: parent_schema_id.to_string(),
                child_schema_id: child_schema_id.to_string(),
            })
            .await?
            .into_inner()
            .relation_id
            .parse()?)
    }

    #[tracing::instrument(skip(self, context))]
    /// Add new object-object edges
    async fn add_edges(
        &self,
        context: &Context<'_>,
        relations: Vec<ObjectRelations>,
    ) -> FieldResult<bool> {
        let mut conn = context.data_unchecked::<EdgeRegistryPool>().get().await?;
        conn.add_edges(rpc::edge_registry::ObjectRelations {
            relations: relations
                .into_iter()
                .map(ObjectRelations::into_edge_rpc)
                .collect(),
        })
        .await?;

        Ok(true)
    }
}
