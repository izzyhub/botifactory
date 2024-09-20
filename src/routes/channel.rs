use axum::{
    routing::{get, post},
    serve::Serve,
    Json, Router,
};

use axum::extract::Path;
use axum::extract::State;
use sqlx::SqlitePool;

pub fn router() -> Router {
    Router::new().route(
        "/project/:project_name/name/:channel_name",
        get(show_project_channel),
    )
}

pub async fn show_project_channel(Path((project_name, channel_name)): Path<(String, String)>) {}
