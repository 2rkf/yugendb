//! Typed collection handle.

use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::{CollectionName, Key, Result, Store, YugenDbError};

/// Typed handle for records in a collection.
#[derive(Clone)]
pub struct Collection<T> {
    store: Store,
    name: CollectionName,
    marker: PhantomData<T>,
}

impl<T> Collection<T> {
    /// Creates a collection handle.
    #[must_use]
    pub fn new(store: Store, name: CollectionName) -> Self {
        Self {
            store,
            name,
            marker: PhantomData,
        }
    }

    /// Returns the collection name.
    #[must_use]
    pub fn name(&self) -> &CollectionName {
        &self.name
    }

    /// Reads a typed value.
    pub async fn get<K>(&self, key: K) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.store.get_from_collection(&self.name, key).await
    }

    /// Writes a typed value.
    pub async fn set<K>(&self, key: K, value: &T) -> Result<()>
    where
        T: Serialize,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.store.set_in_collection(&self.name, key, value).await
    }

    /// Deletes a value.
    pub async fn delete<K>(&self, key: K) -> Result<bool>
    where
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.store.delete_from_collection(&self.name, key).await
    }

    /// Returns whether a key exists.
    pub async fn exists<K>(&self, key: K) -> Result<bool>
    where
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.store.exists_in_collection(&self.name, key).await
    }

    /// Scans typed values by key prefix.
    pub async fn scan_prefix(&self, prefix: impl Into<String>) -> Result<Vec<(Key, T)>>
    where
        T: DeserializeOwned,
    {
        self.store
            .scan_prefix_in_collection(&self.name, prefix)
            .await
    }
}
