//! Stored byte value and metadata types.

use bytes::Bytes;
use chrono::{DateTime, Utc};

/// Opaque serialised value bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueBytes(Bytes);

impl ValueBytes {
    /// Creates value bytes from any byte-like input.
    #[must_use]
    pub fn new(bytes: impl Into<Bytes>) -> Self {
        Self(bytes.into())
    }

    /// Returns the bytes as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns the number of bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true when there are no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts into the inner bytes value.
    #[must_use]
    pub fn into_inner(self) -> Bytes {
        self.0
    }
}

impl From<Vec<u8>> for ValueBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<Bytes> for ValueBytes {
    fn from(value: Bytes) -> Self {
        Self::new(value)
    }
}

impl AsRef<[u8]> for ValueBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Metadata stored alongside a serialised value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueMetadata {
    /// Codec name used to serialise the value.
    pub codec: String,
    /// Creation timestamp in UTC.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp in UTC.
    pub updated_at: DateTime<Utc>,
    /// Expiry timestamp in UTC, when TTL is used.
    pub expires_at: Option<DateTime<Utc>>,
}

impl ValueMetadata {
    /// Creates metadata using the same timestamp for creation and update.
    #[must_use]
    pub fn new(
        codec: impl Into<String>,
        now: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            codec: codec.into(),
            created_at: now,
            updated_at: now,
            expires_at,
        }
    }

    /// Returns true when the metadata expiry time has passed.
    #[must_use]
    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }
}

/// A serialised value and its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredValue {
    /// Serialised value bytes.
    pub bytes: ValueBytes,
    /// Stored value metadata.
    pub metadata: ValueMetadata,
}

impl StoredValue {
    /// Creates a stored value.
    #[must_use]
    pub fn new(bytes: ValueBytes, metadata: ValueMetadata) -> Self {
        Self { bytes, metadata }
    }

    /// Returns true when the value has expired at the current time.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.metadata.is_expired_at(Utc::now())
    }
}
