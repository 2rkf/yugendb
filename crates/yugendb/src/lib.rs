//! User-facing Rust facade crate for yugendb.
//!
//! `yugendb` is the crate Rust applications normally depend on. It re-exports
//! [`yugendb_core`] and exposes optional driver modules behind feature flags.
//!
//! yugendb stores serialised typed values by namespace, collection, and key:
//!
//! ```text
//! namespace + collection + key -> serialised typed value
//! ```
//!
//! SQL drivers use tables, MongoDB uses documents, and Redis uses derived keys
//! behind the driver. Those backend details do not change the public yugendb API.
//!
//! Project-owned terminology uses en-GB spelling, including `initialise`,
//! `finalise`, `serialise`, `deserialise`, and `behaviour`.
//!
//! # Driver features
//!
//! - JSON serialisation is built in through `JsonCodec`.
//! - `memory`: in-process memory driver.
//! - `sqlite`: SQLite driver.
//! - `mysql`: MySQL driver.
//! - `mongodb`: MongoDB driver.
//! - `postgres`: PostgreSQL driver.
//! - `redis`: Redis driver.
//! - `all-drivers`: enables all bundled driver features.
//!
//! External-service drivers require their matching database service at runtime.

pub use yugendb_core::*;

/// Common imports for application code.
pub mod prelude {
    pub use yugendb_core::{
        Batch, BatchOperation, BatchResult, Capabilities, Codec, CodecExt, Collection,
        CollectionName, DeleteOptions, Driver, ErrorCode, JsonCodec, Key, Namespace, ReadOptions,
        Result, ScanOptions, Store, StoreBuilder, StoredValue, Transaction, ValueBytes,
        ValueMetadata, WriteOptions, YugenDbError,
    };
}

/// Feature-gated driver exports.
pub mod drivers {
    /// In-memory driver constructor and type.
    #[cfg(feature = "memory")]
    pub use yugendb_memory::{memory, MemoryDriver};

    /// SQLite driver exports.
    #[cfg(feature = "sqlite")]
    pub mod sqlite {
        pub use yugendb_sqlite::{
            schema_statements, sqlite, sqlite_capabilities, SqliteDriver, SqliteDriverOptions,
            SqliteStorageSchema, ALL_SCHEMA_STATEMENTS, CREATE_EXPIRES_AT_INDEX_SQL,
            CREATE_PREFIX_INDEX_SQL, CREATE_STORE_TABLE_SQL, MIGRATION_METADATA_TABLE_NAME,
            SCHEMA_VERSION, STORE_TABLE_NAME,
        };
    }

    /// MySQL driver exports.
    #[cfg(feature = "mysql")]
    pub mod mysql {
        pub use yugendb_mysql::*;
    }

    /// MongoDB driver exports.
    #[cfg(feature = "mongodb")]
    pub mod mongodb {
        pub use yugendb_mongodb::*;
    }

    /// PostgreSQL driver exports.
    #[cfg(feature = "postgres")]
    pub mod postgres {
        pub use yugendb_postgres::{
            postgres, postgres_capabilities, PostgresDriver, PostgresDriverOptions,
        };
    }

    /// Redis driver exports.
    #[cfg(feature = "redis")]
    pub mod redis {
        pub use yugendb_redis::{redis, redis_capabilities, RedisDriver, RedisDriverOptions};
    }
}
