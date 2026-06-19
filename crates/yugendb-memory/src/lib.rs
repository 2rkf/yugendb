//! In-memory Rust driver for yugendb.
//!
//! The memory driver stores serialised typed values in-process. It is useful
//! for temporary local storage and application-managed caching. It is not a
//! durable storage backend.

use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use parking_lot::RwLock;
use yugendb_core::{
    Batch, BatchOperation, BatchResult, Capabilities, CollectionName, DeleteOptions, Driver, Key,
    Namespace, ReadOptions, Result, ScanOptions, StoredValue, WriteOptions, YugenDbError,
};

/// Convenience constructor for a memory driver.
///
/// The returned value can be passed directly to `Store::builder().driver(...)`.
#[must_use]
pub fn memory() -> MemoryDriver {
    MemoryDriver::new()
}

/// In-process memory driver.
#[derive(Debug, Clone, Default)]
pub struct MemoryDriver {
    entries: Arc<RwLock<BTreeMap<StorageKey, StoredValue>>>,
}

impl MemoryDriver {
    /// Creates an empty memory driver.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of records currently retained by the driver.
    ///
    /// Expired records may still be retained until overwritten, deleted, or the
    /// driver is dropped.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Returns true when no records are currently retained by the driver.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Removes all retained records.
    pub fn clear(&self) {
        self.entries.write().clear();
    }
}

#[async_trait]
impl Driver for MemoryDriver {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::memory()
    }

    async fn initialise(&self) -> Result<()> {
        Ok(())
    }

    async fn finalise(&self) -> Result<()> {
        Ok(())
    }

    async fn get(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &ReadOptions,
    ) -> Result<Option<StoredValue>> {
        let storage_key = StorageKey::new(namespace, collection, key);
        let entries = self.entries.read();
        let Some(value) = entries.get(&storage_key) else {
            return Ok(None);
        };

        if value.is_expired() && !options.allow_expired {
            return Ok(None);
        }

        Ok(Some(value.clone()))
    }

    async fn set(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: StoredValue,
        options: &WriteOptions,
    ) -> Result<()> {
        let storage_key = StorageKey::new(namespace, collection, key);
        let mut entries = self.entries.write();

        if !options.overwrite
            && entries
                .get(&storage_key)
                .is_some_and(|existing| !existing.is_expired())
        {
            return Err(YugenDbError::conflict(
                "memory driver refused to overwrite an existing value",
            ));
        }

        entries.insert(storage_key, value);
        Ok(())
    }

    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool> {
        let storage_key = StorageKey::new(namespace, collection, key);
        let removed = self.entries.write().remove(&storage_key).is_some();

        if options.must_exist && !removed {
            return Err(YugenDbError::not_found(
                "memory driver could not delete a missing value",
            ));
        }

        Ok(removed)
    }

    async fn exists(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
    ) -> Result<bool> {
        let storage_key = StorageKey::new(namespace, collection, key);
        let entries = self.entries.read();
        Ok(entries
            .get(&storage_key)
            .is_some_and(|value| !value.is_expired()))
    }

    async fn scan_prefix(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        options: &ScanOptions,
    ) -> Result<Vec<(Key, StoredValue)>> {
        let prefix = options.prefix.as_deref();
        let entries = self.entries.read();
        let mut results = Vec::new();

        for (storage_key, value) in entries.iter() {
            if storage_key.namespace != namespace.as_str() {
                continue;
            }

            if storage_key.collection != collection.as_str() {
                continue;
            }

            if prefix.is_some_and(|prefix| !storage_key.key.starts_with(prefix)) {
                continue;
            }

            if value.is_expired() && !options.include_expired {
                continue;
            }

            let key = Key::try_from(storage_key.key.as_str())?;
            results.push((key, value.clone()));
        }

        results.sort_by(|(left, _), (right, _)| left.cmp(right));

        if let Some(limit) = options.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn batch(&self, batch: Batch) -> Result<BatchResult> {
        let mut entries = self.entries.write();
        let mut set_count = 0;
        let mut delete_count = 0;

        for operation in batch.into_operations() {
            match operation {
                BatchOperation::Set {
                    namespace,
                    collection,
                    key,
                    value,
                } => {
                    entries.insert(StorageKey::new(&namespace, &collection, &key), value);
                    set_count += 1;
                }
                BatchOperation::Delete {
                    namespace,
                    collection,
                    key,
                } => {
                    if entries
                        .remove(&StorageKey::new(&namespace, &collection, &key))
                        .is_some()
                    {
                        delete_count += 1;
                    }
                }
            }
        }

        Ok(BatchResult::new(set_count, delete_count))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StorageKey {
    namespace: String,
    collection: String,
    key: String,
}

impl StorageKey {
    fn new(namespace: &Namespace, collection: &CollectionName, key: &Key) -> Self {
        Self {
            namespace: namespace.as_str().to_owned(),
            collection: collection.as_str().to_owned(),
            key: key.as_str().to_owned(),
        }
    }
}
