//! SQLite schema helpers for the yugendb storage table.
//!
//! The SQLite driver stores yugendb records through the same core model used by
//! every driver:
//!
//! ```text
//! namespace + collection + key -> serialised typed value
//! ```
//!
//! This module describes the storage schema used by the SQLite driver.

/// Current schema version for the SQLite storage table.
pub const SCHEMA_VERSION: u32 = 1;

/// Main storage table used by the SQLite backend.
pub const STORE_TABLE_NAME: &str = "yugendb_store";

/// Metadata table used for SQLite schema migration bookkeeping.
pub const MIGRATION_METADATA_TABLE_NAME: &str = "yugendb_schema_migrations";

/// SQL statement that creates the main storage table.
pub const CREATE_STORE_TABLE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS yugendb_store (
    namespace TEXT NOT NULL,
    collection TEXT NOT NULL,
    key TEXT NOT NULL,
    value BLOB NOT NULL,
    codec TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT,
    PRIMARY KEY (namespace, collection, key)
);"#;

/// SQL statement that creates the prefix scan index.
pub const CREATE_PREFIX_INDEX_SQL: &str = r#"CREATE INDEX IF NOT EXISTS idx_yugendb_store_prefix
ON yugendb_store(namespace, collection, key);"#;

/// SQL statement that creates the expiry lookup index.
pub const CREATE_EXPIRES_AT_INDEX_SQL: &str = r#"CREATE INDEX IF NOT EXISTS idx_yugendb_store_expires_at
ON yugendb_store(expires_at);"#;

/// SQL statement that creates schema migration bookkeeping.
pub const CREATE_MIGRATION_METADATA_TABLE_SQL: &str = r#"CREATE TABLE IF NOT EXISTS yugendb_schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);"#;

/// All SQL statements needed to initialise the current SQLite schema.
pub const ALL_SCHEMA_STATEMENTS: [&str; 4] = [
    CREATE_STORE_TABLE_SQL,
    CREATE_PREFIX_INDEX_SQL,
    CREATE_EXPIRES_AT_INDEX_SQL,
    CREATE_MIGRATION_METADATA_TABLE_SQL,
];

/// Returns all SQL statements needed to initialise the current SQLite schema.
#[must_use]
pub const fn schema_statements() -> &'static [&'static str] {
    &ALL_SCHEMA_STATEMENTS
}

/// Value object that describes the SQLite storage schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqliteStorageSchema {
    /// Schema version.
    pub version: u32,
    /// Main store table name.
    pub store_table_name: &'static str,
    /// Migration metadata table name.
    pub migration_metadata_table_name: &'static str,
    /// Ordered SQL statements for schema creation.
    pub statements: &'static [&'static str],
}

impl SqliteStorageSchema {
    /// Returns the current SQLite storage schema description.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            version: SCHEMA_VERSION,
            store_table_name: STORE_TABLE_NAME,
            migration_metadata_table_name: MIGRATION_METADATA_TABLE_NAME,
            statements: schema_statements(),
        }
    }
}
