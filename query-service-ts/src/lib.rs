use crate::schema::{Range, SchemaId};
use schema::query_client::QueryClient;
use tonic::transport::Channel;
use utils::query_utils::error::ClientError;

pub mod victoria;

pub mod schema {
    tonic::include_proto!("query");
}

pub async fn connect(addr: String) -> Result<QueryClient<Channel>, ClientError> {
    QueryClient::connect(addr)
        .await
        .map_err(ClientError::ConnectionError)
}

pub async fn query_by_range(
    object_id: String,
    start: String,
    end: String,
    step: String,
    addr: String,
) -> Result<String, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_range(Range {
            object_id,
            start,
            end,
            step,
        })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().timeseries)
}

pub async fn query_by_schema(
    schema_id: String,
    addr: String,
) -> Result<String, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_schema(SchemaId { schema_id })
        .await
        .map_err(ClientError::QueryError)?;

    Ok(response.into_inner().timeseries)
}
