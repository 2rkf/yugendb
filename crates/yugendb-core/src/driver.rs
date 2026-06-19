//! Driver trait for concrete storage backends.

use async_trait::async_trait;

use crate::{
    Batch, BatchResult, Capabilities, CollectionName, DeleteOptions, Key, Namespace, ReadOptions,
    Result, ScanOptions, StoredValue, WriteOptions,
};

/// Async storage driver contract.
#[async_trait]
pub trait Driver: Send + Sync {
    /// Stable driver name.
    fn name(&self) -> &'static str;

    /// Reports supported driver behaviour.
    fn capabilities(&self) -> Capabilities;

    /// Prepares the driver for use.
    async fn initialise(&self) -> Result<()>;

    /// Releases driver resources.
    async fn finalise(&self) -> Result<()>;

    /// Reads a stored value.
    async fn get(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &ReadOptions,
    ) -> Result<Option<StoredValue>>;

    /// Writes a stored value.
    async fn set(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: StoredValue,
        options: &WriteOptions,
    ) -> Result<()>;

    /// Deletes a stored value and returns whether a value was removed.
    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool>;

    /// Returns whether a stored value exists.
    async fn exists(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
    ) -> Result<bool>;

    /// Scans values by key prefix.
    async fn scan_prefix(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        options: &ScanOptions,
    ) -> Result<Vec<(Key, StoredValue)>>;

    /// Applies a batch of write operations.
    async fn batch(&self, batch: Batch) -> Result<BatchResult>;
}
