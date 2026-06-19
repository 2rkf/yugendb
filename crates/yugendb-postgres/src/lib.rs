//! PostgreSQL Rust driver for yugendb.
//!
//! PostgreSQL is a SQL-backed implementation of the yugendb storage model.

pub mod driver;
pub mod schema;

pub use driver::{postgres, postgres_capabilities, PostgresDriver, PostgresDriverOptions};
pub use schema::*;
