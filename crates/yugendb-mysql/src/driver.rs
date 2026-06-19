//! MySQL driver implementation.
//!
//! MySQL is used as a SQL-backed implementation detail for the yugendb storage
//! model. Applications use typed key-value and document-style operations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool, Row};
use yugendb_core::{
    Batch, BatchOperation, BatchResult, Capabilities, CollectionName, DeleteOptions, Driver, Key,
    Namespace, ReadOptions, Result, ScanOptions, StoredValue, ValueBytes, ValueMetadata,
    WriteOptions, YugenDbError,
};

use crate::schema::{schema_statements, MysqlDriverStorageSchema};

/// Options used to configure the MySQL driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MysqlDriverOptions {
    /// Backend connection string.
    pub connection_string: String,
    /// Whether the storage schema should be created during initialisation.
    pub create_schema_on_initialise: bool,
    /// Maximum number of pooled connections.
    pub max_connections: u32,
}

impl MysqlDriverOptions {
    /// Creates default options for a connection string.
    #[must_use]
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            create_schema_on_initialise: true,
            max_connections: 5,
        }
    }
}

/// MySQL driver for durable external-service storage.
#[derive(Debug)]
pub struct MysqlDriver {
    options: MysqlDriverOptions,
    pool: Mutex<Option<MySqlPool>>,
}

impl MysqlDriver {
    /// Creates a new MySQL driver using default options.
    #[must_use]
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self::with_options(MysqlDriverOptions::new(connection_string))
    }

    /// Creates a new MySQL driver with explicit options.
    #[must_use]
    pub fn with_options(options: MysqlDriverOptions) -> Self {
        Self {
            options,
            pool: Mutex::new(None),
        }
    }

    /// Returns the configured options.
    #[must_use]
    pub const fn options(&self) -> &MysqlDriverOptions {
        &self.options
    }

    /// Returns the storage schema.
    #[must_use]
    pub const fn schema(&self) -> MysqlDriverStorageSchema {
        MysqlDriverStorageSchema::current()
    }

    fn map_error(error: sqlx::Error) -> YugenDbError {
        match error {
            sqlx::Error::RowNotFound => YugenDbError::not_found("MySQL row was not found"),
            sqlx::Error::PoolTimedOut | sqlx::Error::Io(_) | sqlx::Error::Tls(_) => {
                YugenDbError::connection_error(error.to_string())
            }
            other => YugenDbError::driver_error("mysql", other.to_string()),
        }
    }

    fn pool(&self) -> Result<MySqlPool> {
        self.pool
            .lock()
            .as_ref()
            .cloned()
            .ok_or_else(|| YugenDbError::connection_error("MySQL driver has not been initialised"))
    }

    fn parse_timestamp(value: String) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&value)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|error| YugenDbError::driver_error("mysql", error.to_string()))
    }

    fn parse_optional_timestamp(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
        value.map(Self::parse_timestamp).transpose()
    }

    fn stored_value_from_row(row: &sqlx::mysql::MySqlRow) -> Result<StoredValue> {
        let bytes: Vec<u8> = row.try_get("value").map_err(Self::map_error)?;
        let codec: String = row.try_get("codec").map_err(Self::map_error)?;
        let created_at: String = row.try_get("created_at").map_err(Self::map_error)?;
        let updated_at: String = row.try_get("updated_at").map_err(Self::map_error)?;
        let expires_at: Option<String> = row.try_get("expires_at").map_err(Self::map_error)?;
        Ok(StoredValue::new(
            ValueBytes::from(bytes),
            ValueMetadata {
                codec,
                created_at: Self::parse_timestamp(created_at)?,
                updated_at: Self::parse_timestamp(updated_at)?,
                expires_at: Self::parse_optional_timestamp(expires_at)?,
            },
        ))
    }

    async fn insert_value(
        executor: impl sqlx::Executor<'_, Database = MySql>,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: &StoredValue,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO yugendb_store
(namespace, collection, `key`, value, codec, created_at, updated_at, expires_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON DUPLICATE KEY UPDATE
    value = VALUES(value),
    codec = VALUES(codec),
    updated_at = VALUES(updated_at),
    expires_at = VALUES(expires_at)"#,
        )
        .bind(namespace.as_str())
        .bind(collection.as_str())
        .bind(key.as_str())
        .bind(value.bytes.as_slice())
        .bind(value.metadata.codec.as_str())
        .bind(value.metadata.created_at.to_rfc3339())
        .bind(value.metadata.updated_at.to_rfc3339())
        .bind(value.metadata.expires_at.as_ref().map(DateTime::to_rfc3339))
        .execute(executor)
        .await
        .map_err(Self::map_error)?;
        Ok(())
    }
}

impl Clone for MysqlDriver {
    fn clone(&self) -> Self {
        Self::with_options(self.options.clone())
    }
}

/// Capabilities for the implemented MySQL driver.
#[must_use]
pub const fn mysql_capabilities() -> Capabilities {
    Capabilities {
        transactions: true,
        ttl: true,
        prefix_scan: true,
        atomic_increment: false,
        batch_write: true,
        raw_sql: false,
        document_query: false,
        json_query: false,
        migrations: true,
        connection_pooling: true,
        watch: false,
        backup: false,
    }
}

/// Convenience constructor for the MySQL driver.
#[must_use]
pub fn mysql(connection_string: impl Into<String>) -> MysqlDriver {
    MysqlDriver::new(connection_string)
}

#[async_trait]
impl Driver for MysqlDriver {
    fn name(&self) -> &'static str {
        "mysql"
    }
    fn capabilities(&self) -> Capabilities {
        mysql_capabilities()
    }

    async fn initialise(&self) -> Result<()> {
        let pool = MySqlPoolOptions::new()
            .max_connections(self.options.max_connections)
            .connect(&self.options.connection_string)
            .await
            .map_err(Self::map_error)?;
        if self.options.create_schema_on_initialise {
            for statement in schema_statements() {
                sqlx::query(statement)
                    .execute(&pool)
                    .await
                    .map_err(Self::map_error)?;
            }
        }
        *self.pool.lock() = Some(pool);
        Ok(())
    }

    async fn finalise(&self) -> Result<()> {
        let pool = {
            let mut guard = self.pool.lock();
            guard.take()
        };

        if let Some(pool) = pool {
            pool.close().await;
        }

        Ok(())
    }

    async fn get(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &ReadOptions,
    ) -> Result<Option<StoredValue>> {
        let pool = self.pool()?;
        let row = sqlx::query("SELECT value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` = ?")
            .bind(namespace.as_str()).bind(collection.as_str()).bind(key.as_str()).fetch_optional(&pool).await.map_err(Self::map_error)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let value = Self::stored_value_from_row(&row)?;
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
        if !options.overwrite
            && self
                .get(namespace, collection, key, &ReadOptions::default())
                .await?
                .is_some()
        {
            return Err(YugenDbError::conflict(
                "MySQL driver refused to overwrite an existing value",
            ));
        }
        let pool = self.pool()?;
        Self::insert_value(&pool, namespace, collection, key, &value).await
    }

    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool> {
        let pool = self.pool()?;
        let removed = sqlx::query(
            "DELETE FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` = ?",
        )
        .bind(namespace.as_str())
        .bind(collection.as_str())
        .bind(key.as_str())
        .execute(&pool)
        .await
        .map_err(Self::map_error)?
        .rows_affected()
            > 0;
        if options.must_exist && !removed {
            return Err(YugenDbError::not_found(
                "MySQL driver could not delete a missing value",
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
        let pool = self.pool()?;
        let prefix = options.prefix.as_deref().unwrap_or("");
        let rows = sqlx::query("SELECT `key`, value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` LIKE CONCAT(?, '%') ORDER BY `key` ASC")
            .bind(namespace.as_str()).bind(collection.as_str()).bind(prefix).fetch_all(&pool).await.map_err(Self::map_error)?;
        let mut values = Vec::new();
        for row in rows {
            let key_text: String = row.try_get("key").map_err(Self::map_error)?;
            let value = Self::stored_value_from_row(&row)?;
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
        let pool = self.pool()?;
        let mut transaction = pool.begin().await.map_err(Self::map_error)?;
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
                    Self::insert_value(&mut *transaction, &namespace, &collection, &key, &value)
                        .await?;
                    set_count += 1;
                }
                BatchOperation::Delete {
                    namespace,
                    collection,
                    key,
                } => {
                    let removed = sqlx::query("DELETE FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` = ?")
                        .bind(namespace.as_str()).bind(collection.as_str()).bind(key.as_str()).execute(&mut *transaction).await.map_err(Self::map_error)?.rows_affected();
                    if removed > 0 {
                        delete_count += 1;
                    }
                }
            }
        }
        transaction.commit().await.map_err(Self::map_error)?;
        Ok(BatchResult::new(set_count, delete_count))
    }
}
