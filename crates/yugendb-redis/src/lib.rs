//! Redis Rust driver for yugendb.
//!
//! Redis is a key-value backend behind the yugendb storage model.

pub mod driver;
pub mod keyspace;

pub use driver::{redis, redis_capabilities, RedisDriver, RedisDriverOptions};
pub use keyspace::*;
