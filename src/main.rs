use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use botifactory::configuration::get_configuration;
use botifactory::run;
use botifactory::telemetry::{get_log_subscriber, init_subscriber};
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");

    let subscriber = get_log_subscriber(
        "botifactory".into(),
        configuration.application.log_level,
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let db_pool = sqlx::SqlitePool::connect_with(configuration.database.with_db()).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    tracing_subscriber::fmt::init();
    println!("Hello, world!");

    run(db_pool, configuration).await;
    Ok(())
}
