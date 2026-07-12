use axum::{
    routing::{get, post},
    Json, Router,
};

use crate::auth::RequireApiToken;
use crate::configuration::Settings;
use crate::routes::error::{conflict_on_unique, APIError, Result};
use crate::validation::ResourceName;
use axum::extract::Path;
use axum::extract::State;
use botifactory_types::{CreateProject, ProjectBody, ProjectJson as Project};
use sqlx::SqlitePool;
use std::fs::create_dir_all;
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
    _auth: RequireApiToken,
    State((db, settings)): State<(SqlitePool, Arc<Settings>)>,
    Json(payload): Json<CreateProject>,
) -> Result<Json<ProjectBody>> {
    // The name ends up as a path component under release_path; ResourceName
    // rejects separators/dots so it cannot escape that directory.
    let project_name = ResourceName::parse(&payload.name)?;

    // Insert before touching the filesystem so a duplicate name is a clean
    // 409 with no orphaned directory.
    sqlx::query!(
        r#"
          insert into projects
          (name) VALUES ($1)
        "#,
        payload.name
    )
    .execute(&db)
    .await
    .map_err(conflict_on_unique)?;

    let project_path = settings
        .application
        .release_path
        .join(project_name.as_str());
    create_dir_all(project_path)?;

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
