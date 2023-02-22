//! Traits and functions to implement CRUD APIs

use std::collections::HashMap;

use anyhow::Context as _;
use tap::prelude::*;
use ulid::Ulid;

use crate::api::ApiError;

pub type Patch = serde_json::Map<String, serde_json::Value>;

pub fn extract_id_from_patch(patch: &Patch) -> Result<Ulid, ApiError> {
    patch
        .get("id")
        .context("Patch is missing `id` field")
        .map_err(ApiError::bad_request)?
        .pipe(serde_json::Value::as_str)
        .context("Invalid type for `id` field")
        .map_err(ApiError::bad_request)?
        .pipe(Ulid::from_string)
        .context("`id` field is not a valid ULID")
        .map_err(ApiError::bad_request)
}

pub trait Key: core::fmt::Display + Sized + core::hash::Hash + Eq {
    fn to_key(&self) -> [u8; 16];
    fn from_key(key: sled::IVec) -> Option<Self>;
}

impl Key for u128 {
    fn to_key(&self) -> [u8; 16] {
        self.to_be_bytes()
    }

    fn from_key(key: sled::IVec) -> Option<Self> {
        <[u8; 16]>::try_from(key.as_ref()).map(u128::from_be_bytes).ok()
    }
}

impl Key for ulid::Ulid {
    fn to_key(&self) -> [u8; 16] {
        u128::from(*self).to_key()
    }

    fn from_key(key: sled::IVec) -> Option<Self> {
        u128::from_key(key).map(ulid::Ulid::from)
    }
}

pub trait Model: serde::de::DeserializeOwned + serde::ser::Serialize {
    const TREE_ID: &'static str;

    fn open_tree(db: &sled::Db) -> sled::Result<sled::Tree> {
        db.open_tree(Self::TREE_ID)
    }
}

pub trait TreeExt {
    fn crud_create<K, M>(&mut self, key: K, value: &M) -> Result<(), ApiError>
    where
        K: Key,
        M: Model;

    fn crud_read<K, M>(&self, keys: K) -> Result<HashMap<K::Item, M>, ApiError>
    where
        K: IntoIterator,
        K::Item: Key,
        M: Model;

    fn crud_read_all<K, M>(&self) -> Result<HashMap<K, M>, ApiError>
    where
        K: Key,
        M: Model;

    fn crud_update<K, M>(&self, key: K, patch: &Patch) -> Result<M, ApiError>
    where
        K: Key,
        M: Model;

    fn crud_delete<K>(&self, keys: K) -> Result<(), ApiError>
    where
        K: IntoIterator,
        K::Item: Key;
}

impl TreeExt for sled::Tree {
    fn crud_create<K, M>(&mut self, key: K, value: &M) -> Result<(), ApiError>
    where
        K: Key,
        M: Model,
    {
        let key = key.to_key();
        let value = bincode::serialize(&value)?;

        if self.contains_key(key)? {
            return Err(ApiError::conflict(anyhow::Error::msg("Already exists")));
        }

        self.insert(key, value)?;

        Ok(())
    }

    fn crud_read<K, M>(&self, keys: K) -> Result<HashMap<K::Item, M>, ApiError>
    where
        K: IntoIterator,
        K::Item: Key,
        M: Model,
    {
        keys.into_iter()
            .map(|key| {
                let val = self
                    .get(key.to_key())?
                    .with_context(|| format!("{key} does not exist"))
                    .map_err(ApiError::not_found)?;
                let val = bincode::deserialize(&val)?;
                Ok((key, val))
            })
            .collect::<Result<HashMap<_, _>, ApiError>>()
    }

    fn crud_read_all<K, M>(&self) -> Result<HashMap<K, M>, ApiError>
    where
        K: Key,
        M: Model,
    {
        self.iter()
            .map(|elem| {
                let (key, val) = elem?;
                let key = K::from_key(key).context("Invalid key").map_err(ApiError::internal)?;
                let val = bincode::deserialize(&val)?;
                Ok((key, val))
            })
            .collect::<Result<HashMap<_, _>, ApiError>>()
    }

    fn crud_update<K, M>(&self, key: K, patch: &Patch) -> Result<M, ApiError>
    where
        K: Key,
        M: Model,
    {
        let mut error = None;

        let update = |current: Option<&[u8]>, patch: &Patch| -> Result<Vec<u8>, ApiError> {
            let current: M = current
                .with_context(|| "{key} does not exist")
                .map_err(ApiError::not_found)?
                .pipe_ref(bincode::deserialize)
                .context("Invalid bincode format")
                .map_err(ApiError::internal)?;

            let mut value: serde_json::Value = serde_json::to_value(current)
                .context("Convert to serde_json::Value")
                .map_err(ApiError::internal)?;

            let value_ref_mut = value.as_object_mut().expect("model");

            for (key, val) in patch {
                value_ref_mut.insert(key.to_owned(), val.to_owned());
            }

            let new_value = serde_json::from_value::<M>(value)?.pipe_ref(bincode::serialize)?;

            Ok(new_value)
        };

        let update_and_fetch_result = self.update_and_fetch(key.to_key(), |current| match update(current, patch) {
            Ok(new_value) => Some(new_value),
            Err(e) => {
                error = Some(e);
                current.map(|v| v.to_vec())
            }
        });

        if let Some(error) = error {
            Err(error)
        } else {
            let result = update_and_fetch_result?.expect("entry");
            let result = bincode::deserialize(&result)
                .context("Invalid bincode format")
                .map_err(ApiError::internal)?;
            Ok(result)
        }
    }

    fn crud_delete<K>(&self, keys: K) -> Result<(), ApiError>
    where
        K: IntoIterator,
        K::Item: Key,
    {
        let mut batch = sled::Batch::default();

        keys.into_iter().for_each(|key| {
            batch.remove(&key.to_key());
        });

        self.apply_batch(batch)?;

        Ok(())
    }
}
