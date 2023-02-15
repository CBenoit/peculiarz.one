use std::collections::HashMap;

use axum::extract::State;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use axum_extra::extract::Query;
use bread_world_models::{Ingredient, Product};
use serde::Deserialize;
use ulid::Ulid;

use crate::api::ApiError;
use crate::crud::{Model, TreeExt};
use crate::AppState;

use super::ApiOk;

impl Model for Ingredient {
    const TREE_ID: &'static str = "bread-world/ingredients";
}

impl Model for Product {
    const TREE_ID: &'static str = "bread-world/products";
}

pub fn make_router(state: AppState) -> Router {
    trace!("Make bread-world router");
    Router::new()
        .route("/ingredients", post(create_ingredient))
        .route("/ingredients", get(read_ingredients))
        .route("/ingredients", patch(update_ingredient))
        .route("/ingredients", delete(delete_ingredients))
        .route("/ingredients/all", get(read_all_ingredients))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(alias = "id")]
    ids: Vec<Ulid>,
}

async fn create_ingredient(State(s): State<AppState>, Json(ingredient): Json<Ingredient>) -> Result<ApiOk, ApiError> {
    Ingredient::open_tree(&s.db)?
        .crud_create(ingredient.id, &ingredient)
        .map(|_| ApiOk)
}

async fn read_ingredients(
    Query(query): Query<ListQuery>,
    State(s): State<AppState>,
) -> Result<Json<HashMap<Ulid, Ingredient>>, ApiError> {
    Ingredient::open_tree(&s.db)?.crud_read(query.ids).map(Json)
}

async fn update_ingredient(State(s): State<AppState>, Json(ingredient): Json<Ingredient>) -> Result<ApiOk, ApiError> {
    Ingredient::open_tree(&s.db)?
        .crud_update(ingredient.id, &ingredient)
        .map(|_| ApiOk)
}

async fn delete_ingredients(Query(query): Query<ListQuery>, State(s): State<AppState>) -> Result<ApiOk, ApiError> {
    Ingredient::open_tree(&s.db)?.crud_delete(query.ids).map(|_| ApiOk)
}

async fn read_all_ingredients(State(s): State<AppState>) -> Result<Json<HashMap<Ulid, Ingredient>>, ApiError> {
    Ingredient::open_tree(&s.db)?.crud_read_all().map(Json)
}
