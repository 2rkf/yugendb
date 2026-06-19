//! User-facing store and builder types.

use std::sync::Arc;

use chrono::Utc;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    Capabilities, Codec, CodecExt, Collection, CollectionName, DeleteOptions, Driver, JsonCodec,
    Key, Namespace, ReadOptions, Result, ScanOptions, StoredValue, ValueMetadata, WriteOptions,
    YugenDbError,
};

/// Main user-facing database handle.
#[derive(Clone)]
pub struct Store {
    driver: Arc<dyn Driver>,
    codec: Arc<dyn Codec>,
    namespace: Namespace,
    default_collection: CollectionName,
}

impl Store {
    /// Creates a store builder.
    #[must_use]
    pub fn builder() -> StoreBuilder {
        StoreBuilder::default()
    }

    /// Creates a store from parts.
    #[must_use]
    pub fn from_parts(
        driver: Arc<dyn Driver>,
        codec: Arc<dyn Codec>,
        namespace: Namespace,
    ) -> Self {
        Self {
            driver,
            codec,
            namespace,
            default_collection: CollectionName::default(),
        }
    }

    /// Prepares the underlying driver for use.
    pub async fn initialise(&self) -> Result<()> {
        self.driver.initialise().await
    }

    /// Releases underlying driver resources.
    pub async fn finalise(&self) -> Result<()> {
        self.driver.finalise().await
    }

    /// Returns the active namespace.
    #[must_use]
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Returns the active driver capabilities.
    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        self.driver.capabilities()
    }

    /// Reads a typed value from the default collection.
    pub async fn get<T, K>(&self, key: K) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.get_from_collection(&self.default_collection, key)
            .await
    }

    /// Writes a typed value to the default collection.
    pub async fn set<T, K>(&self, key: K, value: &T) -> Result<()>
    where
        T: Serialize,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.set_in_collection(&self.default_collection, key, value)
            .await
    }

    /// Deletes a value from the default collection.
    pub async fn delete<K>(&self, key: K) -> Result<bool>
    where
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.delete_from_collection(&self.default_collection, key)
            .await
    }

    /// Returns whether a key exists in the default collection.
    pub async fn exists<K>(&self, key: K) -> Result<bool>
    where
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.exists_in_collection(&self.default_collection, key)
            .await
    }

    /// Scans typed values by key prefix in the default collection.
    pub async fn scan_prefix<T>(&self, prefix: impl Into<String>) -> Result<Vec<(Key, T)>>
    where
        T: DeserializeOwned,
    {
        self.scan_prefix_in_collection(&self.default_collection, prefix)
            .await
    }

    /// Creates a typed collection handle.
    pub fn collection<T, C>(&self, name: C) -> Result<Collection<T>>
    where
        C: TryInto<CollectionName>,
        YugenDbError: From<C::Error>,
    {
        let name = name.try_into()?;
        Ok(Collection::new(self.clone(), name))
    }

    /// Reads a typed value from a specific collection.
    pub async fn get_from_collection<T, K>(
        &self,
        collection: &CollectionName,
        key: K,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        let key = key.try_into()?;
        let options = ReadOptions::default();
        let Some(stored) = self
            .driver
            .get(&self.namespace, collection, &key, &options)
            .await?
        else {
            return Ok(None);
        };

        if stored.is_expired() && !options.allow_expired {
            return Ok(None);
        }

        self.codec.deserialise(&stored.bytes).map(Some)
    }

    /// Writes a typed value to a specific collection.
    pub async fn set_in_collection<T, K>(
        &self,
        collection: &CollectionName,
        key: K,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        self.set_in_collection_with_options(collection, key, value, WriteOptions::default())
            .await
    }

    /// Writes a typed value to a specific collection with options.
    pub async fn set_in_collection_with_options<T, K>(
        &self,
        collection: &CollectionName,
        key: K,
        value: &T,
        options: WriteOptions,
    ) -> Result<()>
    where
        T: Serialize,
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        let key = key.try_into()?;
        let bytes = self.codec.serialise(value)?;
        let now = Utc::now();
        let expires_at = match options.ttl {
            Some(ttl) => {
                let ttl = chrono::Duration::from_std(ttl)
                    .map_err(|_| YugenDbError::invalid_value("ttl duration is too large"))?;
                Some(now + ttl)
            }
            None => None,
        };
        let metadata = ValueMetadata::new(self.codec.name(), now, expires_at);
        let stored = StoredValue::new(bytes, metadata);
        self.driver
            .set(&self.namespace, collection, &key, stored, &options)
            .await
    }

    /// Deletes a value from a specific collection.
    pub async fn delete_from_collection<K>(
        &self,
        collection: &CollectionName,
        key: K,
    ) -> Result<bool>
    where
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        let key = key.try_into()?;
        self.driver
            .delete(&self.namespace, collection, &key, &DeleteOptions::default())
            .await
    }

    /// Returns whether a key exists in a specific collection.
    pub async fn exists_in_collection<K>(&self, collection: &CollectionName, key: K) -> Result<bool>
    where
        K: TryInto<Key>,
        YugenDbError: From<K::Error>,
    {
        let key = key.try_into()?;
        self.driver.exists(&self.namespace, collection, &key).await
    }

    /// Scans typed values by key prefix in a specific collection.
    pub async fn scan_prefix_in_collection<T>(
        &self,
        collection: &CollectionName,
        prefix: impl Into<String>,
    ) -> Result<Vec<(Key, T)>>
    where
        T: DeserializeOwned,
    {
        let options = ScanOptions {
            prefix: Some(prefix.into()),
            ..ScanOptions::default()
        };
        let stored_values = self
            .driver
            .scan_prefix(&self.namespace, collection, &options)
            .await?;

        stored_values
            .into_iter()
            .filter(|(_, stored)| options.include_expired || !stored.is_expired())
            .map(|(key, stored)| {
                self.codec
                    .deserialise(&stored.bytes)
                    .map(|value| (key, value))
            })
            .collect()
    }
}

/// Builder for [`Store`].
#[derive(Clone)]
pub struct StoreBuilder {
    driver: Option<Arc<dyn Driver>>,
    codec: Arc<dyn Codec>,
    namespace: Namespace,
}

impl Default for StoreBuilder {
    fn default() -> Self {
        Self {
            driver: None,
            codec: Arc::new(JsonCodec),
            namespace: Namespace::default(),
        }
    }
}

impl StoreBuilder {
    /// Sets the driver.
    #[must_use]
    pub fn driver<D>(mut self, driver: D) -> Self
    where
        D: Driver + 'static,
    {
        self.driver = Some(Arc::new(driver));
        self
    }

    /// Sets an already shared driver.
    #[must_use]
    pub fn driver_arc(mut self, driver: Arc<dyn Driver>) -> Self {
        self.driver = Some(driver);
        self
    }

    /// Sets the codec.
    #[must_use]
    pub fn codec<C>(mut self, codec: C) -> Self
    where
        C: Codec + 'static,
    {
        self.codec = Arc::new(codec);
        self
    }

    /// Sets the namespace.
    pub fn namespace<N>(mut self, namespace: N) -> Result<Self>
    where
        N: TryInto<Namespace>,
        YugenDbError: From<N::Error>,
    {
        self.namespace = namespace.try_into()?;
        Ok(self)
    }

    /// Builds and initialises the store.
    pub async fn connect(self) -> Result<Store> {
        let store = self.build()?;
        store.initialise().await?;
        Ok(store)
    }

    /// Builds the store without initialising the driver.
    pub fn build(self) -> Result<Store> {
        let Some(driver) = self.driver else {
            return Err(YugenDbError::invalid_value("store driver is required"));
        };

        Ok(Store::from_parts(driver, self.codec, self.namespace))
    }
}
