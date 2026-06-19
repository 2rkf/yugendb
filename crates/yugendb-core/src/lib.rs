//! Core Rust contracts for yugendb.
//!
//! yugendb is a typed key-value and document storage layer. Applications store
//! serialised values by namespace, collection, and key.
//!
//! This crate defines the shared Rust abstractions used by concrete drivers, so
//! application code can stay consistent across supported databases.

pub mod batch;
pub mod capabilities;
pub mod codec;
pub mod collection;
pub mod driver;
pub mod error;
pub mod key;
pub mod options;
pub mod store;
pub mod transaction;
pub mod value;

pub use batch::{Batch, BatchOperation, BatchResult};
pub use capabilities::Capabilities;
pub use codec::{Codec, CodecExt, JsonCodec};
pub use collection::Collection;
pub use driver::Driver;
pub use error::{ErrorCode, Result, YugenDbError};
pub use key::{CollectionName, Key, Namespace};
pub use options::{DeleteOptions, ReadOptions, ScanOptions, WriteOptions};
pub use store::{Store, StoreBuilder};
pub use transaction::Transaction;
pub use value::{StoredValue, ValueBytes, ValueMetadata};
