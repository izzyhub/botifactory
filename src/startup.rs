use sqlx::SqlitePool;

use std::sync::Arc;
use tokio::net::TcpListener;

use crate::configuration::Settings;
use crate::routes::*;
use tracing::{debug};
use tower_http::{trace::TraceLayer, trace};

pub async fn run(db_pool: SqlitePool, settings: Settings) {
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    println!("listening on: {}", address);
    debug!("listening on: {}", address);

    let app = api_router()
        .layer(TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new())
                .on_response(trace::DefaultOnResponse::new())

        )
        .with_state((db_pool, Arc::new(settings)));

    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind address");
    axum::serve(listener, app).await.expect("Failed to serve");
}
