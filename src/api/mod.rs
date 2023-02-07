pub mod bread_world;
pub mod knowledge;

use axum::Router;

pub fn make_router() -> Router {
    Router::new()
        .nest("/bread-world", bread_world::make_router())
        .nest("/knowledge", knowledge::make_router())
}
