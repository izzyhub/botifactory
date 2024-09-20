use axum::{http::StatusCode, routing::get, Router};
use sqlx::SqlitePool;

pub fn router() -> Router<SqlitePool> {
    Router::new().route("/health_check/", get(health_check))
}
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
