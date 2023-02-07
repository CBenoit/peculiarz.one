use axum::{response::IntoResponse, routing::get, Router};

pub fn make_router() -> Router {
    Router::new().route("/recipes", get(get_recipes))
}

async fn get_recipes() -> impl IntoResponse {
    "not yet implemented"
}
