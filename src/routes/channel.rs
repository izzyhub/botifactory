use axum::{
    routing::{get, post},
    Json, Router,
};

use crate::configuration::Settings;
use crate::routes::error::{APIError, Result};
use axum::extract::Path;
use axum::extract::State;
use botifactory_common::{ChannelBody, ChannelJson as Channel, CreateChannel};
use sqlx::SqlitePool;
use std::fs::create_dir_all;
use std::path::PathBuf;
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
    Path(project_name): Path<String>,
    State((db, settings)): State<(SqlitePool, Arc<Settings>)>,
    Json(payload): Json<CreateChannel>,
) -> Result<Json<ChannelBody>> {
    let channel_path: PathBuf = [
        &settings.application.release_path,
        &PathBuf::from(&project_name),
        &PathBuf::from(&payload.channel_name),
    ]
    .iter()
    .collect();
    create_dir_all(channel_path)?;

    sqlx::query!(
        r#"
          insert into release_channel
          (name, project_id) VALUES ($1, (SELECT id from projects where name = $2))
        "#,
        payload.channel_name,
        project_name
    )
    .execute(&db)
    .await?;

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
