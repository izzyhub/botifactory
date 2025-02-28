use axum::{
    routing::{get, post},
    Json, Router,
};

use crate::configuration::Settings;
use crate::routes::error::{APIError, Result};
use axum::extract::Path;
use axum::extract::State;
use botifactory_types::{CreateProject, ProjectBody, ProjectJson as Project};
use sqlx::SqlitePool;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

pub fn router() -> Router<(SqlitePool, Arc<Settings>)> {
    Router::new()
        .route("/project/{name}/", get(show_project))
        .route("/project/{name}", get(show_project))
        .route("/project/new", post(create_project))
        .route("/project/new/", post(create_project))
}
pub async fn show_project(
    Path(project_name): Path<String>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Json<ProjectBody>> {
    let project = sqlx::query_as!(
        Project,
        r#"
          select id,
          name,
          created_at,
          updated_at
          from projects
          where name = $1
        "#,
        project_name
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    Ok(Json(ProjectBody { project }))
}

pub async fn create_project(
    State((db, settings)): State<(SqlitePool, Arc<Settings>)>,
    Json(payload): Json<CreateProject>,
) -> Result<Json<ProjectBody>> {
    let project_path: PathBuf = [
        &settings.application.release_path,
        &PathBuf::from(&payload.name),
    ]
    .iter()
    .collect();
    create_dir_all(project_path)?;
    sqlx::query!(
        r#"
          insert into projects
          (name) VALUES ($1)
        "#,
        payload.name
    )
    .execute(&db)
    .await?;

    let project = sqlx::query_as!(
        Project,
        r#"
          select id,
          name,
          created_at,
          updated_at
          from projects
          where name = $1
        "#,
        payload.name
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    Ok(Json(ProjectBody { project }))
}
