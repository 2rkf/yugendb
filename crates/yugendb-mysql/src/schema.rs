//! MySQL storage schema helpers for yugendb.
//!
//! The schema maps yugendb's namespace, collection, and key contract to one
//! SQL storage table used by the MySQL driver.

/// Storage table used by the MySQL driver.
pub const STORE_TABLE_NAME: &str = "yugendb_store";
/// Current schema version for the MySQL storage table.
pub const SCHEMA_VERSION: u32 = 1;
/// Metadata table used for migration tracking.
pub const MIGRATION_METADATA_TABLE_NAME: &str = "yugendb_migrations";
/// Binary value column type for this backend.
pub const VALUE_COLUMN_TYPE: &str = "LONGBLOB";

/// Table creation statement for the yugendb storage table.
pub const CREATE_STORE_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS yugendb_store (
    namespace VARCHAR(512) NOT NULL,
    collection VARCHAR(512) NOT NULL,
    `key` VARCHAR(512) NOT NULL,
    value LONGBLOB NOT NULL,
    codec TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT,
    PRIMARY KEY (namespace, collection, `key`)
);";

/// Prefix index used by prefix scans where operators create it explicitly.
pub const CREATE_PREFIX_INDEX_SQL: &str = "CREATE INDEX idx_yugendb_store_prefix ON yugendb_store(namespace(191), collection(191), `key`(191));";
/// Expiry index used by TTL filtering where operators create it explicitly.
pub const CREATE_EXPIRES_AT_INDEX_SQL: &str =
    "CREATE INDEX idx_yugendb_store_expires_at ON yugendb_store(expires_at(191));";
/// Ordered schema statements for initial setup.
pub const ALL_SCHEMA_STATEMENTS: [&str; 1] = [CREATE_STORE_TABLE_SQL];

/// Returns all schema statements in execution order.
#[must_use]
pub const fn schema_statements() -> [&'static str; 1] {
    ALL_SCHEMA_STATEMENTS
}

/// Structured schema description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MysqlDriverStorageSchema {
    /// Schema version.
    pub version: u32,
    /// Main storage table.
    pub store_table_name: &'static str,
    /// Migration metadata table.
    pub migration_metadata_table_name: &'static str,
    /// Binary value column type.
    pub value_column_type: &'static str,
}

impl MysqlDriverStorageSchema {
    /// Returns the current schema description.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            version: SCHEMA_VERSION,
            store_table_name: STORE_TABLE_NAME,
            migration_metadata_table_name: MIGRATION_METADATA_TABLE_NAME,
            value_column_type: VALUE_COLUMN_TYPE,
        }
    }
}
