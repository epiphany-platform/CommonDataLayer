use utils::query_utils::error::{ClientError, Status};
use schema_registry::error::RegistryClientError;
use warp::{hyper::StatusCode, reject::Reject, Rejection};

#[derive(Debug)]
pub enum Error {
    RegistryConnectionError(RegistryClientError),
    RegistryError(Status),
    QueryError(ClientError),
    JsonError(serde_json::Error),
    SingleQueryMissingValue,
}

impl Reject for Error {}

impl From<Error> for Rejection {
    fn from(error: Error) -> Rejection {
        warp::reject::custom(error)
    }
}

pub fn recover(rejection: Rejection) -> Result<impl warp::Reply, Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        let message = match error {
            Error::RegistryConnectionError(err) => err.to_string(),
            Error::RegistryError(err) => {
                format!("Error returned from the schema registry: {}", err)
            }
            Error::QueryError(err) => err.to_string(),
            Error::JsonError(err) => format!("Unable to serialize JSON: {}", err),
            Error::SingleQueryMissingValue => "".to_owned(),
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "message": message })),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else {
        Err(rejection)
    }
}
