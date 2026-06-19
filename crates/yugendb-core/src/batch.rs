//! Batch operation types.

use crate::{CollectionName, Key, Namespace, StoredValue};

/// A single batch operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOperation {
    /// Set a stored value.
    Set {
        /// Target namespace.
        namespace: Namespace,
        /// Target collection.
        collection: CollectionName,
        /// Target key.
        key: Key,
        /// Value to store.
        value: StoredValue,
    },
    /// Delete a stored value.
    Delete {
        /// Target namespace.
        namespace: Namespace,
        /// Target collection.
        collection: CollectionName,
        /// Target key.
        key: Key,
    },
}

impl BatchOperation {
    /// Creates a set operation.
    #[must_use]
    pub fn set(
        namespace: Namespace,
        collection: CollectionName,
        key: Key,
        value: StoredValue,
    ) -> Self {
        Self::Set {
            namespace,
            collection,
            key,
            value,
        }
    }

    /// Creates a delete operation.
    #[must_use]
    pub fn delete(namespace: Namespace, collection: CollectionName, key: Key) -> Self {
        Self::Delete {
            namespace,
            collection,
            key,
        }
    }
}

/// Ordered batch of write operations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    operations: Vec<BatchOperation>,
}

impl Batch {
    /// Creates an empty batch.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an operation to the batch.
    pub fn push(&mut self, operation: BatchOperation) {
        self.operations.push(operation);
    }

    /// Adds an operation and returns the batch for chaining.
    #[must_use]
    pub fn with_operation(mut self, operation: BatchOperation) -> Self {
        self.push(operation);
        self
    }

    /// Returns all operations.
    #[must_use]
    pub fn operations(&self) -> &[BatchOperation] {
        &self.operations
    }

    /// Consumes the batch and returns the operations.
    #[must_use]
    pub fn into_operations(self) -> Vec<BatchOperation> {
        self.operations
    }

    /// Returns true when the batch has no operations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Returns the number of operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }
}

/// Summary returned after a batch write.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatchResult {
    /// Number of set operations applied.
    pub set_count: usize,
    /// Number of delete operations applied.
    pub delete_count: usize,
}

impl BatchResult {
    /// Creates a batch result.
    #[must_use]
    pub const fn new(set_count: usize, delete_count: usize) -> Self {
        Self {
            set_count,
            delete_count,
        }
    }

    /// Returns the total number of affected operations.
    #[must_use]
    pub const fn total_count(self) -> usize {
        self.set_count + self.delete_count
    }
}
