//! SQLite Rust driver for yugendb.
//!
//! SQLite is an embedded SQL-backed implementation of yugendb's storage model:
//!
//! ```text
//! namespace + collection + key -> serialised typed value
//! ```

pub mod driver;
pub mod schema;

pub use driver::{sqlite, sqlite_capabilities, SqliteDriver, SqliteDriverOptions};
pub use schema::{
    schema_statements, SqliteStorageSchema, ALL_SCHEMA_STATEMENTS, CREATE_EXPIRES_AT_INDEX_SQL,
    CREATE_MIGRATION_METADATA_TABLE_SQL, CREATE_PREFIX_INDEX_SQL, CREATE_STORE_TABLE_SQL,
    MIGRATION_METADATA_TABLE_NAME, SCHEMA_VERSION, STORE_TABLE_NAME,
};
