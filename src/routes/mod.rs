pub mod channel;
pub mod error;
pub mod health_check;
pub mod project;
pub mod release;

use crate::configuration::Settings;
use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;

pub fn api_router() -> Router<(SqlitePool, Arc<Settings>)> {
    project::router()
        .merge(health_check::router())
        .merge(channel::router())
        .merge(release::router())
}
