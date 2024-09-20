use axum::{
    routing::{get, post},
    serve::Serve,
    Json, Router,
};

use axum::extract::Path;
use axum::extract::State;
use sqlx::SqlitePool;

pub fn router() -> Router {
    Router::new()
        .route(
            "/project/:project_name/name/:channel_name/latest",
            get(show_latest_project_release),
        )
        .route(
            "/project/:project_name/name/:channel_name/previous",
            get(show_previous_project_release),
        )
        .route(
            "/project/:project_name/name/:channel_name/:id",
            get(show_project_release),
        )
        .route(
            "/project/:project_name/name/:channel_name/new",
            post(create_project_release),
        )
}

pub async fn show_latest_project_release(
    Path((project_name, channel_name)): Path<(String, String)>,
) {
}

pub async fn show_previous_project_release(
    Path((project_name, channel_name)): Path<(String, String)>,
) {
}

pub async fn show_project_release(
    Path((project_name, channel_name, id)): Path<(String, String, u64)>,
) {
}

pub async fn create_project_release(Path((project_name, channel_name)): Path<(String, String)>) {}
