//! Transaction trait for drivers that support transaction-like behaviour.

use async_trait::async_trait;

use crate::{
    CollectionName, DeleteOptions, Key, Namespace, ReadOptions, Result, StoredValue, WriteOptions,
};

/// Basic async transaction contract.
///
/// High-level callback helpers will be added after real drivers prove the
/// exact ergonomics. Drivers that do not support transactions should report
/// that through capabilities and return `UNSUPPORTED_FEATURE` for transaction
/// entry points.
#[async_trait]
pub trait Transaction: Send + Sync {
    /// Reads a value inside the transaction.
    async fn get(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &ReadOptions,
    ) -> Result<Option<StoredValue>>;

    /// Writes a value inside the transaction.
    async fn set(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: StoredValue,
        options: &WriteOptions,
    ) -> Result<()>;

    /// Deletes a value inside the transaction.
    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool>;

    /// Commits the transaction.
    async fn commit(self: Box<Self>) -> Result<()>;

    /// Rolls back the transaction.
    async fn rollback(self: Box<Self>) -> Result<()>;
}
