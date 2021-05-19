use crate::error::ClientError;
use query_service_ts_client::QueryServiceTsClient;
use tonic::transport::Channel;

pub use crate::codegen::query_service_ts::*;

pub async fn connect(addr: String) -> Result<QueryServiceTsClient<Channel>, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<QueryServiceTsClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(QueryServiceTsClient::with_interceptor(
        conn,
        tracing_utils::grpc::interceptor(),
    ))
}

pub async fn query_by_range(
    schema_id: String,
    object_id: String,
    start: String,
    end: String,
    step: String,
    addr: String,
) -> Result<String, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_range(Range {
            schema_id,
            object_id,
            start,
            end,
            step,
        })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().timeseries)
}

pub async fn query_by_schema(schema_id: String, addr: String) -> Result<String, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_schema(SchemaId { schema_id })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().timeseries)
}

pub async fn query_raw(raw_statement: String, addr: String) -> Result<Vec<u8>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_raw(RawStatement { raw_statement })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().value_bytes)
}
