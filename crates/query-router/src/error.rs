use rpc::error::ClientError;
use warp::{hyper::StatusCode, reject::Reject, Rejection};

#[derive(Debug)]
pub enum Error {
    ClientError(ClientError),
    JsonError(serde_json::Error),
    SingleQueryMissingValue,
    RawQueryMissingValue,
    WrongValueFormat,
}

impl Reject for Error {}

pub fn recover(rejection: Rejection) -> Result<impl warp::Reply, Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        let message = match error {
            Error::ClientError(err) => err.to_string(),
            Error::JsonError(err) => format!("Unable to serialize JSON: {}", err),
            Error::SingleQueryMissingValue => "".to_owned(),
            Error::WrongValueFormat => "".to_owned(),
            Error::RawQueryMissingValue => "".to_owned(),
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "message": message })),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else {
        Err(rejection)
    }
}
