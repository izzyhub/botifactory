use crate::configuration::Settings;
use axum::{http::StatusCode, routing::get, Router};
use sqlx::SqlitePool;
use std::sync::Arc;

pub fn router() -> Router<(SqlitePool, Arc<Settings>)> {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/health-check", get(health_check))
        .route("/healthcheck", get(health_check))
        .route("/healthcheck/", get(health_check))
}
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
