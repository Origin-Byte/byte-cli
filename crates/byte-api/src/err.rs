use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use rust_sdk::err::RustSdkError;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteApiError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    RustSdkError(#[from] RustSdkError),
}

impl fmt::Display for ByteApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ResponseError for ByteApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ByteApiError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ByteApiError::SerdeJson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ByteApiError::SerdeYaml(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ByteApiError::AnyhowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ByteApiError::RustSdkError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(format!("{}", self))
    }
}
