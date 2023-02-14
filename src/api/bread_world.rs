use anyhow::Context as _;
use axum::extract::State;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use axum_extra::extract::Query;
use bread_world_models::Ingredient;
use serde::Deserialize;
use ulid::Ulid;

use crate::api::ApiError;
use crate::AppState;

pub fn make_router(state: AppState) -> Router {
    trace!("Make bread-world router");
    Router::new()
        .route("/ingredients", post(post_ingredient))
        .route("/ingredients", get(get_ingredients))
        .route("/ingredients", patch(patch_ingredient))
        .route("/ingredients", delete(delete_ingredients))
        .route("/ingredients/all", get(get_all_ingredients))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQuery {
    id: Vec<Ulid>,
}

async fn post_ingredient(State(s): State<AppState>, Json(ingredient): Json<Ingredient>) -> Result<(), ApiError> {
    let key = u128::from(ingredient.id).to_be_bytes();
    let value = bincode::serialize(&ingredient)?;

    let tree = s.db.open_tree("ingredients")?;

    if tree.contains_key(&key)? {
        return Err(ApiError::conflict(anyhow::Error::msg(
            "An ingredient with the same ID already exists",
        )));
    }

    tree.insert(key, value)?;

    Ok(())
}

async fn get_ingredients(
    Query(query): Query<ListQuery>,
    State(s): State<AppState>,
) -> Result<Json<Vec<Ingredient>>, ApiError> {
    let tree = s.db.open_tree("ingredients")?;

    let ingredients = query
        .id
        .into_iter()
        .map(|id| {
            let key = u128::from(id).to_be_bytes();
            tree.get(key)?
                .with_context(|| format!("{} not found", id))
                .map_err(ApiError::not_found)
        })
        .map(|val| {
            let val = val?;
            let val = bincode::deserialize(&val)?;
            Ok(val)
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(ingredients))
}

async fn patch_ingredient(State(s): State<AppState>, Json(ingredient): Json<Ingredient>) -> Result<(), ApiError> {
    let key = u128::from(ingredient.id).to_be_bytes();
    let value = bincode::serialize(&ingredient)?;

    let tree = s.db.open_tree("ingredients")?;
    tree.insert(key, value)?;

    Ok(())
}

async fn delete_ingredients(
    Query(query): Query<ListQuery>,
    State(s): State<AppState>,
) -> Result<Json<Vec<Ingredient>>, ApiError> {
    let tree = s.db.open_tree("ingredients")?;

    let ingredients = query
        .id
        .into_iter()
        .map(|id| {
            let key = u128::from(id).to_be_bytes();
            tree.remove(key)?
                .with_context(|| format!("{} not found", id))
                .map_err(ApiError::not_found)
        })
        .map(|val| {
            let val = val?;
            let val = bincode::deserialize(&val)?;
            Ok(val)
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(ingredients))
}

async fn get_all_ingredients(State(s): State<AppState>) -> Result<Json<Vec<Ingredient>>, ApiError> {
    let tree = s.db.open_tree("ingredients")?;

    let ingredients = tree
        .iter()
        .map(|elem| {
            let val = elem?.1;
            let val = bincode::deserialize(&val)?;
            Ok(val)
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(ingredients))
}
