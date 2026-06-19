//! SQLite driver implementation.
//!
//! SQLite stores yugendb records through the shared storage-table model:
//! `namespace + collection + key -> serialised typed value`.
//! Applications use the same yugendb store and collection API across backends.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use yugendb_core::{
    Batch, BatchOperation, BatchResult, Capabilities, CollectionName, DeleteOptions, Driver, Key,
    Namespace, ReadOptions, Result, ScanOptions, StoredValue, ValueBytes, ValueMetadata,
    WriteOptions, YugenDbError,
};

use crate::schema::{schema_statements, SqliteStorageSchema};

/// Options used to configure the SQLite driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqliteDriverOptions {
    /// Filesystem path or SQLite connection target.
    pub path: String,
    /// Whether the database file should be created if it is missing.
    pub create_if_missing: bool,
}

impl SqliteDriverOptions {
    /// Creates default SQLite options for a path.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            create_if_missing: true,
        }
    }
}

/// SQLite driver for durable local storage.
#[derive(Debug)]
pub struct SqliteDriver {
    options: SqliteDriverOptions,
    connection: Mutex<Option<Connection>>,
}

impl SqliteDriver {
    /// Creates a new SQLite driver using default options.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self::with_options(SqliteDriverOptions::new(path))
    }

    /// Creates a new SQLite driver with explicit options.
    #[must_use]
    pub fn with_options(options: SqliteDriverOptions) -> Self {
        Self {
            options,
            connection: Mutex::new(None),
        }
    }

    /// Returns the configured options.
    #[must_use]
    pub const fn options(&self) -> &SqliteDriverOptions {
        &self.options
    }

    /// Returns the SQLite storage schema description.
    #[must_use]
    pub const fn schema(&self) -> SqliteStorageSchema {
        SqliteStorageSchema::current()
    }

    fn map_error(error: rusqlite::Error) -> YugenDbError {
        YugenDbError::driver_error("sqlite", error.to_string())
    }

    fn with_connection<T>(&self, operation: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self.connection.lock();
        let Some(connection) = guard.as_ref() else {
            return Err(YugenDbError::connection_error(
                "SQLite driver has not been initialised",
            ));
        };
        operation(connection)
    }

    fn parse_timestamp(value: String) -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&value)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|error| YugenDbError::driver_error("sqlite", error.to_string()))
    }

    fn parse_optional_timestamp(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
        value.map(Self::parse_timestamp).transpose()
    }

    fn stored_value_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredValue> {
        let bytes: Vec<u8> = row.get("value")?;
        let codec: String = row.get("codec")?;
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;
        let expires_at: Option<String> = row.get("expires_at")?;

        let created_at = Self::parse_timestamp(created_at).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let updated_at = Self::parse_timestamp(updated_at).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let expires_at = Self::parse_optional_timestamp(expires_at).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;

        Ok(StoredValue::new(
            ValueBytes::from(bytes),
            ValueMetadata {
                codec,
                created_at,
                updated_at,
                expires_at,
            },
        ))
    }

    fn insert_value(
        connection: &Connection,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: &StoredValue,
    ) -> Result<()> {
        connection
            .execute(
                r#"INSERT INTO yugendb_store
(namespace, collection, key, value, codec, created_at, updated_at, expires_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
ON CONFLICT(namespace, collection, key) DO UPDATE SET
    value = excluded.value,
    codec = excluded.codec,
    updated_at = excluded.updated_at,
    expires_at = excluded.expires_at"#,
                params![
                    namespace.as_str(),
                    collection.as_str(),
                    key.as_str(),
                    value.bytes.as_slice(),
                    value.metadata.codec.as_str(),
                    value.metadata.created_at.to_rfc3339(),
                    value.metadata.updated_at.to_rfc3339(),
                    value
                        .metadata
                        .expires_at
                        .as_ref()
                        .map(|expires_at| expires_at.to_rfc3339()),
                ],
            )
            .map_err(Self::map_error)?;
        Ok(())
    }
}

impl Clone for SqliteDriver {
    fn clone(&self) -> Self {
        // Cloned handles are configured the same way but own a separate lazy connection.
        Self::with_options(self.options.clone())
    }
}

/// Capabilities for the implemented SQLite driver.
#[must_use]
pub const fn sqlite_capabilities() -> Capabilities {
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
        connection_pooling: false,
        watch: false,
        backup: false,
    }
}


/// Convenience constructor for the SQLite driver.
#[must_use]
pub fn sqlite(path: impl Into<String>) -> SqliteDriver {
    SqliteDriver::new(path)
}

#[async_trait]
impl Driver for SqliteDriver {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn capabilities(&self) -> Capabilities {
        sqlite_capabilities()
    }

    async fn initialise(&self) -> Result<()> {
        let mut guard = self.connection.lock();
        if guard.is_some() {
            return Ok(());
        }

        let connection = if self.options.create_if_missing {
            Connection::open(&self.options.path)
        } else {
            Connection::open_with_flags(
                &self.options.path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            )
        }
        .map_err(Self::map_error)?;

        for statement in schema_statements() {
            connection
                .execute_batch(statement)
                .map_err(Self::map_error)?;
        }

        *guard = Some(connection);
        Ok(())
    }

    async fn finalise(&self) -> Result<()> {
        let mut guard = self.connection.lock();
        *guard = None;
        Ok(())
    }

    async fn get(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &ReadOptions,
    ) -> Result<Option<StoredValue>> {
        self.with_connection(|connection| {
            let value = connection
                .query_row(
                    "SELECT value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ?1 AND collection = ?2 AND key = ?3",
                    params![namespace.as_str(), collection.as_str(), key.as_str()],
                    Self::stored_value_from_row,
                )
                .optional()
                .map_err(Self::map_error)?;

            Ok(value.filter(|value| options.allow_expired || !value.is_expired()))
        })
    }

    async fn set(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        value: StoredValue,
        options: &WriteOptions,
    ) -> Result<()> {
        self.with_connection(|connection| {
            if !options.overwrite {
                let existing = connection
                    .query_row(
                        "SELECT value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ?1 AND collection = ?2 AND key = ?3",
                        params![namespace.as_str(), collection.as_str(), key.as_str()],
                        Self::stored_value_from_row,
                    )
                    .optional()
                    .map_err(Self::map_error)?;

                if existing.is_some_and(|existing| !existing.is_expired()) {
                    return Err(YugenDbError::conflict(
                        "SQLite driver refused to overwrite an existing value",
                    ));
                }
            }

            Self::insert_value(connection, namespace, collection, key, &value)
        })
    }

    async fn delete(
        &self,
        namespace: &Namespace,
        collection: &CollectionName,
        key: &Key,
        options: &DeleteOptions,
    ) -> Result<bool> {
        self.with_connection(|connection| {
            let removed = connection
                .execute(
                    "DELETE FROM yugendb_store WHERE namespace = ?1 AND collection = ?2 AND key = ?3",
                    params![namespace.as_str(), collection.as_str(), key.as_str()],
                )
                .map_err(Self::map_error)?
                > 0;

            if options.must_exist && !removed {
                return Err(YugenDbError::not_found(
                    "SQLite driver could not delete a missing value",
                ));
            }

            Ok(removed)
        })
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
        self.with_connection(|connection| {
            let prefix = options.prefix.as_deref().unwrap_or("");
            let mut statement = connection
                .prepare(
                    "SELECT key, value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ?1 AND collection = ?2 AND key LIKE ?3 ORDER BY key ASC",
                )
                .map_err(Self::map_error)?;
            let rows = statement
                .query_map(
                    params![namespace.as_str(), collection.as_str(), format!("{prefix}%")],
                    |row| {
                        let key: String = row.get("key")?;
                        let value = Self::stored_value_from_row(row)?;
                        Ok((key, value))
                    },
                )
                .map_err(Self::map_error)?;

            let mut values = Vec::new();
            for row in rows {
                let (key, value) = row.map_err(Self::map_error)?;
                if value.is_expired() && !options.include_expired {
                    continue;
                }
                values.push((Key::try_from(key)?, value));
                if let Some(limit) = options.limit {
                    if values.len() >= limit {
                        break;
                    }
                }
            }
            Ok(values)
        })
    }

    async fn batch(&self, batch: Batch) -> Result<BatchResult> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction().map_err(Self::map_error)?;
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
                        Self::insert_value(&transaction, &namespace, &collection, &key, &value)?;
                        set_count += 1;
                    }
                    BatchOperation::Delete {
                        namespace,
                        collection,
                        key,
                    } => {
                        let removed = transaction
                            .execute(
                                "DELETE FROM yugendb_store WHERE namespace = ?1 AND collection = ?2 AND key = ?3",
                                params![namespace.as_str(), collection.as_str(), key.as_str()],
                            )
                            .map_err(Self::map_error)?;
                        if removed > 0 {
                            delete_count += 1;
                        }
                    }
                }
            }

            transaction.commit().map_err(Self::map_error)?;
            Ok(BatchResult::new(set_count, delete_count))
        })
    }
}
