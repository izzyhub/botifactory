use axum::{
    routing::{get, post},
    serve::Serve,
    Json, Router,
};

use axum::extract::Path;
use axum::extract::State;
use sqlx::SqlitePool;

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/project/:name/", get(show_project))
        .route("/project/new", post(create_project))
}
pub async fn show_project(Path(project_name): Path<String>, State(db): State<SqlitePool>) {}
pub async fn create_project(State(db): State<SqlitePool>) {}
