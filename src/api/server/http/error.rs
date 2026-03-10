use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum HttpError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    InternalError(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            HttpError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            HttpError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            HttpError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            HttpError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            HttpError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            HttpError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            HttpError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

impl From<crate::api::core::CoreError> for HttpError {
    fn from(err: crate::api::core::CoreError) -> Self {
        use crate::api::core::CoreError;
        match err {
            CoreError::NotFound(msg) => HttpError::NotFound(msg),
            CoreError::InvalidParameter(msg) => HttpError::BadRequest(msg),
            CoreError::QueryExecutionFailed(msg) => HttpError::InternalError(msg),
            CoreError::TransactionFailed(msg) => HttpError::InternalError(msg),
            CoreError::SchemaOperationFailed(msg) => HttpError::InternalError(msg),
            CoreError::StorageError(msg) => HttpError::InternalError(msg),
            CoreError::Internal(msg) => HttpError::InternalError(msg),
            CoreError::DetailedQueryError { message, .. } => HttpError::InternalError(message),
        }
    }
}
