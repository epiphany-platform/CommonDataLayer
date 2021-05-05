use crate::error::ClientError;
use query_service_client::QueryServiceClient;
use std::collections::HashMap;
use tonic::transport::Channel;

pub use crate::codegen::query_service::*;

pub async fn connect(addr: String) -> Result<QueryServiceClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<QueryServiceClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(QueryServiceClient::with_interceptor(
        conn,
        tracing_tools::grpc::interceptor(),
    ))
}

pub async fn query_multiple(
    object_ids: Vec<String>,
    addr: String,
) -> Result<HashMap<String, Vec<u8>>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_multiple(ObjectIds { object_ids })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

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
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().values)
}

pub async fn query_raw(raw_statement: String, addr: String) -> Result<Vec<u8>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_raw(RawStatement { raw_statement })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().value_bytes)
}
