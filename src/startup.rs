use sqlx::SqlitePool;

use std::sync::Arc;
use tokio::net::TcpListener;

use crate::configuration::Settings;
use crate::routes::*;

pub async fn run(db_pool: SqlitePool, settings: Settings) {
    println!("Hello, world!");
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );

    let app = api_router().with_state((db_pool, Arc::new(settings)));

    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind address");
    axum::serve(listener, app).await.expect("Failed to serve");
}
