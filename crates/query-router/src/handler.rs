use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use warp::hyper::header::CONTENT_TYPE;

use crate::error::Error;
use crate::schema::{SchemaCache, SchemaMetadata};
use futures_util::TryStreamExt;
use rpc::schema_registry::types::SchemaType;
use rpc::{query_service, query_service_ts};
use settings_utils::RepositoryStaticRouting;

const APPLICATION_JSON: &str = "application/json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Body {
    Range {
        from: String,
        to: String,
        step: String,
    },
    Raw {
        raw_statement: String,
    },
    Empty {},
}

#[tracing::instrument(skip(cache))]
pub async fn query_single(
    object_id: Uuid,
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
    request_body: Body,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = get_routing_info(schema_id, repository_id, cache, routing).await?;

    let values = match (&schema_type, request_body) {
        (SchemaType::DocumentStorage, _) => {
            let mut values: HashMap<String, Vec<u8>> = rpc::query_service::query_multiple(
                vec![object_id.to_string()],
                query_address.clone(),
            )
            .await
            .map_err(Error::ClientError)?
            .map_ok(|object| (object.object_id, object.payload))
            .try_collect()
            .await
            .map_err(Error::ClientError)?;

            values
                .remove(&object_id.to_string())
                .ok_or(Error::SingleQueryMissingValue)
        }

        (SchemaType::Timeseries, Body::Range { from, to, step }) => {
            let timeseries = rpc::query_service_ts::query_by_range(
                schema_id.to_string(),
                object_id.to_string(),
                from,
                to,
                step,
                query_address.clone(),
            )
            .await
            .map_err(Error::ClientError)?;

            Ok(timeseries.into_bytes())
        }

        (SchemaType::Timeseries, Body::Empty {}) => Err(Error::SingleQueryMissingValue),
        (_, Body::Raw { raw_statement: _ }) => Err(Error::WrongValueFormat),
    }?;

    Ok(warp::reply::with_header(
        values,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}

#[tracing::instrument(skip(cache))]
pub async fn query_multiple(
    object_ids: String,
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = get_routing_info(schema_id, repository_id, cache, routing).await?;

    let object_ids = object_ids.split(',').map(str::to_owned).collect();

    let values = match schema_type {
        SchemaType::DocumentStorage => {
            let values: HashMap<_, _> =
                rpc::query_service::query_multiple(object_ids, query_address.clone())
                    .await
                    .map_err(Error::ClientError)?
                    .map_ok(|o| (o.object_id, o.payload))
                    .try_collect()
                    .await
                    .map_err(Error::ClientError)?;

            values
        }
        _ => {
            return Err(warp::Rejection::from(Error::ExpectedSchemaType(
                SchemaType::DocumentStorage,
            )));
        }
    };

    Ok(warp::reply::with_header(
        serde_json::to_vec(&byte_map_to_json_map(values)?).map_err(Error::JsonError)?,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}

#[tracing::instrument(skip(cache))]
pub async fn query_by_schema(
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = get_routing_info(schema_id, repository_id, cache, routing).await?;

    match &schema_type {
        SchemaType::DocumentStorage => {
            let values: HashMap<_, _> =
                rpc::query_service::query_by_schema(schema_id.to_string(), query_address.clone())
                    .await
                    .map_err(Error::ClientError)?
                    .map_ok(|o| (o.object_id, o.payload))
                    .try_collect()
                    .await
                    .map_err(Error::ClientError)?;

            Ok(warp::reply::with_header(
                serde_json::to_vec(&byte_map_to_json_map(values)?).map_err(Error::JsonError)?,
                CONTENT_TYPE,
                APPLICATION_JSON,
            ))
        }
        SchemaType::Timeseries => {
            let timeseries = rpc::query_service_ts::query_by_schema(
                schema_id.to_string(),
                query_address.clone(),
            )
            .await
            .map_err(Error::ClientError)?;
            Ok(warp::reply::with_header(
                timeseries.into_bytes(),
                CONTENT_TYPE,
                APPLICATION_JSON,
            ))
        }
    }
}

#[tracing::instrument(skip(cache))]
pub async fn query_raw(
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    request_body: Body,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = get_routing_info(schema_id, repository_id, cache, routing).await?;

    let values = match (request_body, &schema_type) {
        (Body::Raw { raw_statement }, SchemaType::DocumentStorage) => {
            query_service::query_raw(raw_statement, query_address.clone())
                .await
                .map_err(Error::ClientError)
        }

        (Body::Raw { raw_statement }, SchemaType::Timeseries) => {
            query_service_ts::query_raw(raw_statement, query_address.clone())
                .await
                .map_err(Error::ClientError)
        }

        (Body::Empty {}, _) => Err(Error::RawQueryMissingValue),

        (Body::Range { .. }, _) => Err(Error::WrongValueFormat),
    }?;

    Ok(warp::reply::with_header(
        values,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}

fn byte_map_to_json_map(map: HashMap<String, Vec<u8>>) -> Result<HashMap<String, Value>, Error> {
    map.into_iter()
        .map(|(object_id, value)| {
            Ok((
                object_id,
                serde_json::from_slice(&value).map_err(Error::JsonError)?,
            ))
        })
        .collect()
}

async fn get_routing_info(
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<SchemaMetadata, Error> {
    let metadata = if let Some(repository_id) = repository_id {
        let entry = routing.get(&repository_id);
        if let Some(routing) = entry {
            SchemaMetadata {
                query_address: routing.query_address.clone(),
                schema_type: routing.repository_type,
            }
        } else {
            return Err(Error::InvalidRepository(repository_id));
        }
    } else {
        cache
            .get(schema_id)
            .await
            .map_err(Error::SchemaFetchError)?
    };

    Ok(metadata)
}
