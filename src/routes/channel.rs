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
use botifactory_types::{ChannelBody, ChannelJson as Channel, CreateChannel};
use sqlx::SqlitePool;
use std::fs::create_dir_all;
use std::sync::Arc;

pub fn router() -> Router<(SqlitePool, Arc<Settings>)> {
    Router::new()
        .route("/{project_name}/{channel_name}", get(show_project_channel))
        .route("/channel/{channel_id}", get(show_channel_by_id))
        .route("/{project_name}/channel/new", post(create_project_channel))
}

pub async fn show_project_channel(
    Path((project_name, channel_name)): Path<(String, String)>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Json<ChannelBody>> {
    let channel = sqlx::query_as!(
        Channel,
        r#"
          select id,
          name,
          project_id,
          created_at,
          updated_at
          from release_channel
          where name = $1
          and project_id = 
          (select id from projects where name = $2)
        "#,
        channel_name,
        project_name
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    Ok(Json(ChannelBody { channel }))
}
pub async fn show_channel_by_id(
    Path(channel_id): Path<i64>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Json<ChannelBody>> {
    let channel = sqlx::query_as!(
        Channel,
        r#"
          select id,
          name,
          project_id,
          created_at,
          updated_at
          from release_channel
          where id = $1
        "#,
        channel_id,
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    Ok(Json(ChannelBody { channel }))
}
pub async fn create_project_channel(
    _auth: RequireApiToken,
    Path(project_name): Path<String>,
    State((db, settings)): State<(SqlitePool, Arc<Settings>)>,
    Json(payload): Json<CreateChannel>,
) -> Result<Json<ChannelBody>> {
    // Both names end up as path components under release_path; ResourceName
    // rejects separators/dots so they cannot escape that directory.
    let project_name_validated = ResourceName::parse(&project_name)?;
    let channel_name = ResourceName::parse(&payload.channel_name)?;

    // Look the project up explicitly so a missing project is a 404 rather
    // than a NOT NULL constraint failure (500).
    let project_id = sqlx::query_scalar!(
        r#"select id from projects where name = $1"#,
        project_name
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    // Insert before touching the filesystem so a duplicate channel is a
    // clean 409 with no orphaned directory.
    sqlx::query!(
        r#"
          insert into release_channel
          (name, project_id) VALUES ($1, $2)
        "#,
        payload.channel_name,
        project_id
    )
    .execute(&db)
    .await
    .map_err(conflict_on_unique)?;

    let channel_path = settings
        .application
        .release_path
        .join(project_name_validated.as_str())
        .join(channel_name.as_str());
    create_dir_all(channel_path)?;

    let channel = sqlx::query_as!(
        Channel,
        r#"
          select id,
          name,
          project_id,
          created_at,
          updated_at
          from release_channel
          where name = $1
          and project_id = 
          (select id from projects where name = $2)
        "#,
        payload.channel_name,
        project_name
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    Ok(Json(ChannelBody { channel }))
}
