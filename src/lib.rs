#[macro_use]
extern crate tracing;

pub mod api;
pub mod config;

use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use config::ArcConfig;
use tokio::fs;

#[derive(Clone)]
pub struct AppState {
    pub db: sled::Db,
    pub config: ArcConfig,
}

pub fn make_router(state: AppState) -> Router {
    Router::new().route("/bread-world", get(bread_world)).with_state(state)
}

pub async fn bread_world(State(s): State<AppState>) -> impl IntoResponse {
    let content = fs::read_to_string(s.config.assets_dir.join("bread-world.html"))
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}
