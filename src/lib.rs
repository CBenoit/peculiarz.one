pub mod api;
pub mod config;

use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use config::ArcConfig;
use tokio::fs;

pub fn make_router(config: ArcConfig) -> Router {
    Router::new().route("/bread-world", get(bread_world)).with_state(config)
}

pub async fn bread_world(State(config): State<ArcConfig>) -> impl IntoResponse {
    let content = fs::read_to_string(config.assets_dir.join("bread-world.html"))
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(content)
        .unwrap()
}
