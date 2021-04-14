use crate::error::Error;
use rpc::schema_registry::types::SchemaType;
use rpc::{query_service, query_service_ts};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryInto;
use uuid::Uuid;
use warp::hyper::header::CONTENT_TYPE;

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

#[tracing::instrument(skip(schema_registry_addr))]
pub async fn query_single(
    object_id: Uuid,
    schema_id: Uuid,
    schema_registry_addr: String,
    request_body: Body,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = rpc::schema_registry::connect(schema_registry_addr)
        .await
        .map_err(Error::ClientError)?;
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id {
            id: schema_id.to_string(),
        })
        .await
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?
        .into_inner();
    let schema_type: rpc::schema_registry::types::SchemaType = schema
        .schema_type
        .try_into()
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?;

    let values = match (&schema_type, request_body) {
        (SchemaType::DocumentStorage, _) => {
            let mut values = rpc::query_service::query_multiple(
                vec![object_id.to_string()],
                schema.query_address.clone(),
            )
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
                schema.query_address.clone(),
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

#[tracing::instrument(skip(schema_registry_addr))]
pub async fn query_multiple(
    object_ids: String,
    schema_id: Uuid,
    schema_registry_addr: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = rpc::schema_registry::connect(schema_registry_addr)
        .await
        .map_err(Error::ClientError)?;
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id {
            id: schema_id.to_string(),
        })
        .await
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?
        .into_inner();

    let object_ids = object_ids.split(',').map(str::to_owned).collect();
    let values = rpc::query_service::query_multiple(object_ids, schema.query_address.clone())
        .await
        .map_err(Error::ClientError)?;

    Ok(warp::reply::with_header(
        serde_json::to_vec(&byte_map_to_json_map(values)?).map_err(Error::JsonError)?,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}

#[tracing::instrument(skip(schema_registry_addr))]
pub async fn query_by_schema(
    schema_id: Uuid,
    schema_registry_addr: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = rpc::schema_registry::connect(schema_registry_addr)
        .await
        .map_err(Error::ClientError)?;
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id {
            id: schema_id.to_string(),
        })
        .await
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?
        .into_inner();
    let schema_type: rpc::schema_registry::types::SchemaType = schema
        .schema_type
        .try_into()
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?;

    match &schema_type {
        SchemaType::DocumentStorage => {
            let values = rpc::query_service::query_by_schema(
                schema_id.to_string(),
                schema.query_address.clone(),
            )
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
                schema.query_address.clone(),
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

#[tracing::instrument(skip(schema_registry_addr))]
pub async fn query_raw(
    schema_id: Uuid,
    schema_registry_addr: String,
    request_body: Body,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = rpc::schema_registry::connect(schema_registry_addr)
        .await
        .map_err(Error::ClientError)?;
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id {
            id: schema_id.to_string(),
        })
        .await
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?
        .into_inner();
    let schema_type: rpc::schema_registry::types::SchemaType = schema
        .schema_type
        .try_into()
        .map_err(|err| Error::ClientError(rpc::error::schema_registry_error(err)))?;

    let values = match (request_body, &schema_type) {
        (Body::Raw { raw_statement }, SchemaType::DocumentStorage) => {
            query_service::query_raw(raw_statement, schema.query_address.clone())
                .await
                .map_err(Error::ClientError)
        }

        (Body::Raw { raw_statement }, SchemaType::Timeseries) => {
            query_service_ts::query_raw(raw_statement, schema.query_address.clone())
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
