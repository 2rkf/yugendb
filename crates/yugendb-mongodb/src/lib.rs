//! MongoDB Rust driver for yugendb.
//!
//! MongoDB is a document-backed implementation of yugendb's namespace,
//! collection, and key contract.

pub mod document;
pub mod driver;

pub use document::*;
pub use driver::{mongodb, mongodb_capabilities, MongoDbDriver, MongoDbDriverOptions};
