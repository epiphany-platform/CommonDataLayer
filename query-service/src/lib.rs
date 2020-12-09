use error::ClientError;
use schema::{query_client::QueryClient, ObjectIds, RawStatement, SchemaId};
use std::collections::HashMap;
use tonic::transport::Channel;
use utils::query_utils::error::ClientError;

pub mod druid;
pub mod ds;
pub mod psql;

pub mod schema {
    tonic::include_proto!("query");
}

pub async fn connect(addr: String) -> Result<QueryClient<Channel>, ClientError> {
    QueryClient::connect(addr)
        .await
        .map_err(ClientError::ConnectionError)
}

pub async fn query_multiple(
    object_ids: Vec<String>,
    addr: String,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_multiple(ObjectIds { object_ids })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().values)
}

pub async fn query_by_schema(
    schema_id: String,
    addr: String,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_schema(SchemaId { schema_id })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().values)
}

pub async fn query_raw(raw_statement: String, addr: String) -> Result<Vec<u8>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_raw(RawStatement { raw_statement })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().value_bytes)
}
