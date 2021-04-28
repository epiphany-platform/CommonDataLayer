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
    InvalidSchemaType(rpc::tonic::Status),
    ExpectedSchemaType(SchemaType),
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
        };

        let code = match error {
            Error::ExpectedSchemaType(_) => StatusCode::BAD_REQUEST,
            Error::ClientError(_)
            | Error::JsonError(_)
            | Error::SingleQueryMissingValue
            | Error::RawQueryMissingValue
            | Error::WrongValueFormat
            | Error::InvalidSchemaType(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "message": message })),
            code,
        ))
    } else {
        Err(rejection)
    }
}
