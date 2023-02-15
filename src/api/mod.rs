pub mod bread_world;
pub mod knowledge;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};

use crate::AppState;

pub fn make_router(state: AppState) -> Router {
    Router::new()
        .nest("/bread-world", bread_world::make_router(state))
        .nest("/knowledge", knowledge::make_router())
}

pub struct ApiError {
    status_code: StatusCode,
    source: anyhow::Error,
}

impl ApiError {
    pub fn bad_request(source: anyhow::Error) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            source,
        }
    }

    pub fn internal(source: anyhow::Error) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            source,
        }
    }

    pub fn not_found(source: anyhow::Error) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            source,
        }
    }

    pub fn conflict(source: anyhow::Error) -> Self {
        Self {
            status_code: StatusCode::CONFLICT,
            source,
        }
    }
}

impl<E> From<E> for ApiError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(e: E) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            source: anyhow::Error::new(e),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "status": self.status_code.as_u16(),
            "details": format!("{:?}", self.source),
        });

        let mut response = Json(body).into_response();
        *response.status_mut() = self.status_code;

        response
    }
}

pub struct ApiOk;

impl IntoResponse for ApiOk {
    fn into_response(self) -> Response {
        Json(serde_json::json!({
            "status": 200,
            "details": "OK",
        }))
        .into_response()
    }
}
