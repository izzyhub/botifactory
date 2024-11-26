use axum::body::Body;
use axum::http::{header::InvalidHeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use tracing;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    error: String,
}

impl ErrorBody {
    fn new(error: String) -> Self {
        ErrorBody { error }
    }
}

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Internal Error")]
    InternalError,
    #[error("Request Error")]
    RequestError,
    #[error("Not found")]
    NotFound,
    #[error("Unsupported content type")]
    UnsupportedMediaType,
    #[error("Database error")]
    SqlError(#[from] sqlx::Error),
    #[error("Filesystem error")]
    FsError(#[from] std::io::Error),
    #[error("Bad header value")]
    HeaderError(#[from] InvalidHeaderValue),
    #[error("Bad header value")]
    PersistBinaryError(#[from] tempfile::PersistError),
}

pub type Result<T, E = APIError> = std::result::Result<T, E>;

impl IntoResponse for APIError {
    fn into_response(self) -> Response<Body> {
        let (status_code, error) = match self {
            Self::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Error".to_string(),
            ),
            Self::RequestError => (StatusCode::BAD_REQUEST, "Bad Request".to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Self::UnsupportedMediaType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported media type. Supported types are 'application/octet-stream' and 'application/json'".to_string()),
            Self::HeaderError(internal_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Bad header: {internal_error}"),
            ),
            Self::SqlError(internal_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database Error: {internal_error}"),
            ),
            Self::FsError(internal_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Filesystem Error: {internal_error}"),
            ),
            Self::PersistBinaryError(internal_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Persisting temporary file error: {internal_error}"),
            ),
        };

        tracing::error!(error);
        let error_body = ErrorBody::new(error.to_string());
        (status_code, Json(error_body)).into_response()
    }
}
