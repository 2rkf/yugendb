//! MySQL Rust driver for yugendb.
//!
//! MySQL is a SQL-backed implementation of the yugendb storage model.

pub mod driver;
pub mod schema;

pub use driver::{mysql, mysql_capabilities, MysqlDriver, MysqlDriverOptions};
pub use schema::*;
