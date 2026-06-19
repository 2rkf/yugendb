//! Operation option types.

use std::time::Duration;

/// Options for read operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOptions {
    /// When true, a driver may return expired values.
    pub allow_expired: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            allow_expired: false,
        }
    }
}

/// Options for write operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteOptions {
    /// Optional time-to-live duration.
    pub ttl: Option<Duration>,
    /// When false, setting an existing key should fail with a conflict.
    pub overwrite: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            ttl: None,
            overwrite: true,
        }
    }
}

/// Options for delete operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteOptions {
    /// When true, deleting a missing key should fail.
    pub must_exist: bool,
}

impl Default for DeleteOptions {
    fn default() -> Self {
        Self { must_exist: false }
    }
}

/// Options for prefix scans.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanOptions {
    /// Prefix to match against keys.
    pub prefix: Option<String>,
    /// Maximum number of records to return.
    pub limit: Option<usize>,
    /// When true, a driver may include expired values.
    pub include_expired: bool,
}
