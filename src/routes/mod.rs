pub mod channel;
pub mod health_check;
pub mod project;
pub mod release;

use axum::Router;
use sqlx::SqlitePool;

pub fn api_router() -> Router<SqlitePool> {
    project::router()
        .merge(health_check::router())
        .merge(channel::router())
        .merge(release::router())
}
