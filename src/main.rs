use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context as _;
use axum::http::StatusCode;
use axum::routing::get_service;
use axum::Router;
use peculiarzone::config::Config;
use tap::prelude::*;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|arg| arg == "-h" || arg == "--help") {
        Config::show_help();
        return Ok(());
    }

    dotenvy::dotenv().context("Failed to load .env file")?;

    let config = Config::from_env().pipe(Arc::new);

    // enable console logging
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .nest("/api", peculiarzone::api::make_router())
        .merge(peculiarzone::make_router(config.clone()))
        .route_service(
            "/*path",
            get_service(ServeDir::new(&config.assets_dir)).handle_error(|e| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {e}"),
                )
            }),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::new(config.addr, config.port);
    tracing::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");

    Ok(())
}
