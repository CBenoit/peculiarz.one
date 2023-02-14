use anyhow::Context as _;
use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use bread_world_models::Ingredient;
use ulid::Ulid;

use crate::api::ApiError;
use crate::AppState;

pub fn make_router(state: AppState) -> Router {
    trace!("Make bread-world router");
    Router::new()
        .route("/ingredients", get(list_ingredients))
        .route("/ingredients", patch(patch_ingredient))
        .route("/ingredients", post(post_ingredient))
        .route("/ingredients/:id", get(get_ingredient))
        .route("/ingredients/:id", delete(delete_ingredient))
        .with_state(state)
}

pub async fn list_ingredients(State(s): State<AppState>) -> Result<Json<Vec<Ingredient>>, ApiError> {
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

pub async fn get_ingredient(Path(id): Path<Ulid>, State(s): State<AppState>) -> Result<Json<Ingredient>, ApiError> {
    let key = u128::from(id).to_be_bytes();

    let tree = s.db.open_tree("ingredients")?;
    let value = tree
        .get(key)?
        .context("Ingredient not found")
        .map_err(ApiError::not_found)?;

    let ingredient: Ingredient = bincode::deserialize(&value)?;

    Ok(Json(ingredient))
}

pub async fn delete_ingredient(Path(id): Path<Ulid>, State(s): State<AppState>) -> Result<Json<Ingredient>, ApiError> {
    let key = u128::from(id).to_be_bytes();

    let tree = s.db.open_tree("ingredients")?;
    let value = tree
        .remove(key)?
        .context("Ingredient not found")
        .map_err(ApiError::not_found)?;

    let ingredient: Ingredient = bincode::deserialize(&value)?;

    Ok(Json(ingredient))
}

pub async fn post_ingredient(State(s): State<AppState>, Json(ingredient): Json<Ingredient>) -> Result<(), ApiError> {
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

pub async fn patch_ingredient(State(s): State<AppState>, Json(ingredient): Json<Ingredient>) -> Result<(), ApiError> {
    let key = u128::from(ingredient.id).to_be_bytes();
    let value = bincode::serialize(&ingredient)?;

    let tree = s.db.open_tree("ingredients")?;
    tree.insert(key, value)?;

    Ok(())
}
