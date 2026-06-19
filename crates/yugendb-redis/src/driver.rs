//! Redis driver implementation.
//!
//! Redis stores each yugendb record as a JSON envelope at a derived Redis key.
//! Redis remains an implementation detail behind the namespace, collection, and
//! key contract.

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use yugendb_core::{
    Batch, BatchOperation, BatchResult, Capabilities, CollectionName, DeleteOptions, Driver, Key,
    Namespace, ReadOptions, Result, ScanOptions, StoredValue, ValueBytes, ValueMetadata,
    WriteOptions, YugenDbError,
};

use crate::keyspace::{redis_key, redis_prefix, RedisKeyspace};

/// Options used to configure the Redis driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisDriverOptions {
    /// Redis connection string.
    pub connection_string: String,
    /// Prefix added to all Redis keys created by the driver.
    pub key_prefix: String,
}

impl RedisDriverOptions {
    /// Creates default options for a Redis connection string.
    #[must_use]
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            key_prefix: "yugendb".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RedisEnvelope {
    value: String,
    codec: String,
    created_at: String,
    updated_at: String,
    expires_at: Option<String>,
}

/// Redis driver for external key-value storage.
#[derive(Debug, Clone)]
pub struct RedisDriver {
    options: RedisDriverOptions,
    client: redis::Client,
}

impl RedisDriver {
    /// Creates a new Redis driver using default options.
    pub fn new(connection_string: impl Into<String>) -> Result<Self> {
        Self::with_options(RedisDriverOptions::new(connection_string))
    }

    /// Creates a new Redis driver with explicit options.
    pub fn with_options(options: RedisDriverOptions) -> Result<Self> {
        let client = redis::Client::open(options.connection_string.as_str())
            .map_err(|error| YugenDbError::connection_error(error.to_string()))?;
        Ok(Self { options, client })
    }

    /// Returns the configured options.
    #[must_use]
    pub const fn options(&self) -> &RedisDriverOptions {
        &self.options
    }

    /// Returns keyspace helper information.
    #[must_use]
    pub fn keyspace(&self) -> RedisKeyspace {
        RedisKeyspace::new(self.options.key_prefix.clone())
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| YugenDbError::connection_error(error.to_string()))
    }

    fn map_error(error: redis::RedisError) -> YugenDbError {
        if error.is_timeout() {
            YugenDbError::timeout(error.to_string())
        } else {
            YugenDbError::driver_error("redis", error.to_string())
        }
    }

    fn key(&self, namespace: &Namespace, collection: &CollectionName, key: &Key) -> String {
        redis_key(
            &self.options.key_prefix,
            namespace.as_str(),
            collection.as_str(),
            key.as_str(),
        )
    }

    fn prefix(&self, namespace: &Namespace, collection: &CollectionName, prefix: &str) -> String {
        redis_prefix(
            &self.options.key_prefix,
            namespace.as_str(),
            collection.as_str(),
            prefix,
        )
    }

    fn envelope_from_value(value: &StoredValue) -> RedisEnvelope {
        RedisEnvelope {
            value: BASE64.encode(value.bytes.as_slice()),
            codec: value.metadata.codec.clone(),
            created_at: value.metadata.created_at.to_rfc3339(),
            updated_at: value.metadata.updated_at.to_rfc3339(),
            expires_at: value.metadata.expires_at.as_ref().map(DateTime::to_rfc3339),
        }
    }

    fn value_from_envelope(envelope: RedisEnvelope) -> Result<StoredValue> {
        let bytes = BASE64
            .decode(envelope.value)
            .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?;
        let created_at = DateTime::parse_from_rfc3339(&envelope.created_at)
            .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&envelope.updated_at)
            .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?
            .with_timezone(&Utc);
        let expires_at = envelope
            .expires_at
            .map(|value| {
                DateTime::parse_from_rfc3339(&value).map(|value| value.with_timezone(&Utc))
            })
            .transpose()
            .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?;
        Ok(StoredValue::new(
            ValueBytes::from(bytes),
            ValueMetadata {
                codec: envelope.codec,
                created_at,
                updated_at,
                expires_at,
            },
        ))
    }

    async fn write_value(&self, redis_key: &str, value: &StoredValue) -> Result<()> {
        let mut connection = self.connection().await?;
        let envelope = serde_json::to_string(&Self::envelope_from_value(value))
            .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?;
        connection
            .set::<_, _, ()>(redis_key, envelope)
            .await
            .map_err(Self::map_error)?;
        if let Some(expires_at) = value.metadata.expires_at {
            let ttl_ms = (expires_at - Utc::now()).num_milliseconds().max(1);
            redis::cmd("PEXPIRE")
                .arg(redis_key)
                .arg(ttl_ms)
                .query_async::<()>(&mut connection)
                .await
                .map_err(Self::map_error)?;
        }
        Ok(())
    }
}

/// Capabilities for the implemented Redis driver.
#[must_use]
pub const fn redis_capabilities() -> Capabilities {
    Capabilities {
        transactions: false,
        ttl: true,
        prefix_scan: true,
        atomic_increment: false,
        batch_write: true,
        raw_sql: false,
        document_query: false,
        json_query: false,
        migrations: false,
        connection_pooling: false,
        watch: false,
        backup: false,
    }
}

/// Convenience constructor for the Redis driver.
pub fn redis(connection_string: impl Into<String>) -> Result<RedisDriver> {
    RedisDriver::new(connection_string)
}

#[async_trait]
impl Driver for RedisDriver {
    fn name(&self) -> &'static str {
        "redis"
    }
    fn capabilities(&self) -> Capabilities {
        redis_capabilities()
    }
    async fn initialise(&self) -> Result<()> {
        let mut connection = self.connection().await?;
        redis::cmd("PING")
            .query_async::<String>(&mut connection)
            .await
            .map_err(Self::map_error)?;
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
        let redis_key = self.key(namespace, collection, key);
        let mut connection = self.connection().await?;
        let raw: Option<String> = connection.get(redis_key).await.map_err(Self::map_error)?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let envelope: RedisEnvelope = serde_json::from_str(&raw)
            .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?;
        let value = Self::value_from_envelope(envelope)?;
        Ok((options.allow_expired || !value.is_expired()).then_some(value))
    }

    async fn set(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: StoredValue,
        options: &WriteOptions,
    ) -> Result<()> {
        let redis_key = self.key(namespace, collection, key);
        if !options.overwrite && self.exists(namespace, collection, key).await? {
            return Err(YugenDbError::conflict(
                "Redis driver refused to overwrite an existing value",
            ));
        }
        self.write_value(&redis_key, &value).await
    }

    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool> {
        let redis_key = self.key(namespace, collection, key);
        let mut connection = self.connection().await?;
        let removed: usize = connection.del(redis_key).await.map_err(Self::map_error)?;
        if options.must_exist && removed == 0 {
            return Err(YugenDbError::not_found(
                "Redis driver could not delete a missing value",
            ));
        }
        Ok(removed > 0)
    }

    async fn exists(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
    ) -> Result<bool> {
        Ok(self
            .get(namespace, collection, key, &ReadOptions::default())
            .await?
            .is_some())
    }

    async fn scan_prefix(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        options: &ScanOptions,
    ) -> Result<Vec<(Key, StoredValue)>> {
        let prefix = options.prefix.as_deref().unwrap_or("");
        let pattern = format!("{}*", self.prefix(namespace, collection, prefix));
        let mut connection = self.connection().await?;
        let mut cursor = 0_u64;
        let mut redis_keys = Vec::new();
        loop {
            let (next, mut keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .cursor_arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut connection)
                .await
                .map_err(Self::map_error)?;
            redis_keys.append(&mut keys);
            cursor = next;
            if cursor == 0 {
                break;
            }
        }
        redis_keys.sort();
        let mut values = Vec::new();
        for redis_key in redis_keys {
            let Some(key_text) = crate::keyspace::decode_redis_key(
                &self.options.key_prefix,
                namespace.as_str(),
                collection.as_str(),
                &redis_key,
            ) else {
                continue;
            };
            let raw: Option<String> = connection.get(&redis_key).await.map_err(Self::map_error)?;
            let Some(raw) = raw else {
                continue;
            };
            let value = Self::value_from_envelope(
                serde_json::from_str(&raw)
                    .map_err(|error| YugenDbError::driver_error("redis", error.to_string()))?,
            )?;
            if value.is_expired() && !options.include_expired {
                continue;
            }
            values.push((Key::try_from(key_text)?, value));
            if let Some(limit) = options.limit {
                if values.len() >= limit {
                    break;
                }
            }
        }
        Ok(values)
    }

    async fn batch(&self, batch: Batch) -> Result<BatchResult> {
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
                    self.set(
                        &namespace,
                        &collection,
                        &key,
                        value,
                        &WriteOptions::default(),
                    )
                    .await?;
                    set_count += 1;
                }
                BatchOperation::Delete {
                    namespace,
                    collection,
                    key,
                } => {
                    if self
                        .delete(&namespace, &collection, &key, &DeleteOptions::default())
                        .await?
                    {
                        delete_count += 1;
                    }
                }
            }
        }
        Ok(BatchResult::new(set_count, delete_count))
    }
}
