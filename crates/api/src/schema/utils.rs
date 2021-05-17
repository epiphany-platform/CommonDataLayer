use crate::schema::context::SchemaRegistryConn;
use crate::types::schema::FullSchema;
use crate::types::view::FullView;
use async_graphql::FieldResult;
use uuid::Uuid;

pub async fn get_view(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<FullView> {
    tracing::debug!("get view: {:?}", id);
    let view = conn
        .get_view(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(|source| rpc::error::ClientError::QueryError { source })?
        .into_inner();

    FullView::from_rpc(view)
}

pub async fn get_schema(conn: &mut SchemaRegistryConn, id: Uuid) -> FieldResult<FullSchema> {
    tracing::debug!("get schema: {:?}", id);
    let schema = conn
        .get_full_schema(rpc::schema_registry::Id { id: id.to_string() })
        .await
        .map_err(|source| rpc::error::ClientError::QueryError { source })?
        .into_inner();

    FullSchema::from_rpc(schema)
}
