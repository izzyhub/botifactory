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
    println!("reading configuration");
    let configuration = get_configuration().expect("Failed to read configuration");
    println!("read configuration");

    let subscriber = get_log_subscriber(
        "botifactory".into(),
        configuration.application.log_level,
        std::io::stdout,
    );
    init_subscriber(subscriber);
    println!("created log sbuscriber");

    println!("creating sqlite connection");
    let db_pool = sqlx::SqlitePool::connect_with(configuration.database.with_db()).await?;
    println!("connected to sqlite");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    println!("Ran migrations");

    //tracing_subscriber::fmt::init();
    println!("Hello, world!");

    run(db_pool, configuration).await;
    Ok(())
}
