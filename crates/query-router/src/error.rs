use rpc::error::ClientError;
use rpc::schema_registry::types::SchemaType;
use warp::{hyper::StatusCode, reject::Reject, Rejection};

#[derive(Debug)]
pub enum Error {
    ClientError(ClientError),
    JsonError(serde_json::Error),
    SingleQueryMissingValue,
    RawQueryMissingValue,
    WrongValueFormat,
    InvalidSchemaType(anyhow::Error),
    ExpectedSchemaType(SchemaType),
    InvalidRepository(String),
    SchemaFetchError(anyhow::Error),
}

impl Reject for Error {}

pub fn recover(rejection: Rejection) -> Result<impl warp::Reply, Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        let message = match error {
            Error::ClientError(err) => err.to_string(),
            Error::JsonError(err) => format!("Unable to serialize JSON: {}", err),
            Error::SingleQueryMissingValue => "Value not returned from query".to_owned(),
            Error::WrongValueFormat => "Value incorrectly formatted".to_owned(),
            Error::RawQueryMissingValue => "Value not returned from query".to_owned(),
            Error::InvalidSchemaType(err) => format!("Failed to parse schema type: {}", err),
            Error::ExpectedSchemaType(expected) => {
                format!("This route expects schema type: {:?}", expected)
            }
            Error::InvalidRepository(repository_id) => format!(
                "Repository with id `{}` is not present in config",
                repository_id
            ),
            Error::SchemaFetchError(error) => format!("Failed to fetch schema: {}", error),
        };

        let code = match error {
            Error::ExpectedSchemaType(_) | Error::InvalidRepository(_) => StatusCode::BAD_REQUEST,
            Error::ClientError(_)
            | Error::JsonError(_)
            | Error::SingleQueryMissingValue
            | Error::RawQueryMissingValue
            | Error::WrongValueFormat
            | Error::InvalidSchemaType(_)
            | Error::SchemaFetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "message": message })),
            code,
        ))
    } else {
        Err(rejection)
    }
}
