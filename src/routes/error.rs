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
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Conflict")]
    Conflict,
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
    #[error("Semver error")]
    VersionParseError(#[from] semver::Error),
    #[error("Validation error")]
    ValidationError(#[from] crate::validation::ValidationError),
}

/// Maps a unique-constraint violation to 409 Conflict; everything else stays
/// a database error.
pub fn conflict_on_unique(error: sqlx::Error) -> APIError {
    match &error {
        sqlx::Error::Database(db_error) if db_error.is_unique_violation() => APIError::Conflict,
        _ => APIError::SqlError(error),
    }
}

pub type Result<T, E = APIError> = std::result::Result<T, E>;

impl IntoResponse for APIError {
    fn into_response(self) -> Response<Body> {
        // Internal errors (database, filesystem, headers) are logged with full
        // detail but reported to the client generically: the detail includes
        // schema and filesystem paths that an unauthenticated caller should
        // not see.
        let (status_code, error) = match self {
            Self::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Error".to_string(),
            ),
            Self::RequestError => (StatusCode::BAD_REQUEST, "Bad Request".to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            Self::Conflict => (StatusCode::CONFLICT, "Conflict".to_string()),
            Self::UnsupportedMediaType => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported media type. Supported types are 'application/octet-stream' and 'application/json'".to_string()),
            Self::HeaderError(internal_error) => {
                tracing::error!("header error: {internal_error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error".to_string())
            }
            Self::SqlError(internal_error) => {
                tracing::error!("database error: {internal_error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string())
            }
            Self::FsError(internal_error) => {
                tracing::error!("filesystem error: {internal_error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Filesystem Error".to_string())
            }
            Self::PersistBinaryError(internal_error) => {
                tracing::error!("failed to persist uploaded file: {internal_error}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error".to_string())
            }
            Self::VersionParseError(internal_error) => (
                StatusCode::BAD_REQUEST,
                format!("Bad version format: {internal_error}"),
            ),
            Self::ValidationError(validation_error) => (
                StatusCode::BAD_REQUEST,
                format!("Validation error: {validation_error}"),
            ),
        };

        tracing::error!(error);
        let error_body = ErrorBody::new(error.to_string());
        (status_code, Json(error_body)).into_response()
    }
}
