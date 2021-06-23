pub mod edges;
pub mod ingestion;
pub mod materialization;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct RequestError(String);

impl RequestError {
    fn new(error: impl Into<String>) -> Self {
        Self(error.into())
    }
}

impl From<anyhow::Error> for RequestError {
    fn from(err: anyhow::Error) -> Self {
        Self(format!("{:?}", err))
    }
}

impl From<uuid::Error> for RequestError {
    fn from(err: uuid::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(err: serde_json::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<std::num::TryFromIntError> for RequestError {
    fn from(err: std::num::TryFromIntError) -> Self {
        Self(err.to_string())
    }
}

impl From<RequestError> for tonic::Status {
    fn from(err: RequestError) -> Self {
        tonic::Status::invalid_argument(err.0)
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ResponseError(String);

impl From<serde_json::Error> for ResponseError {
    fn from(err: serde_json::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<ResponseError> for tonic::Status {
    fn from(err: ResponseError) -> Self {
        tonic::Status::internal(err.0)
    }
}

type RequestResult<T> = std::result::Result<T, RequestError>;
type ResponseResult<T> = std::result::Result<T, ResponseError>;
