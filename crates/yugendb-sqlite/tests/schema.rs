use yugendb_core::Driver;
use yugendb_sqlite::{
    schema_statements, sqlite, sqlite_capabilities, SqliteDriver,
    CREATE_EXPIRES_AT_INDEX_SQL, CREATE_PREFIX_INDEX_SQL, CREATE_STORE_TABLE_SQL, STORE_TABLE_NAME,
};

#[test]
fn schema_statements_include_expected_sql() {
    let statements = schema_statements();

    assert!(statements
        .iter()
        .any(|statement| statement.contains("CREATE TABLE")));
    assert!(statements
        .iter()
        .any(|statement| statement.contains(STORE_TABLE_NAME)));
    assert!(statements
        .iter()
        .any(|statement| statement.contains("idx_yugendb_store_prefix")));
    assert!(statements
        .iter()
        .any(|statement| statement.contains("idx_yugendb_store_expires_at")));
}

#[test]
fn schema_constants_match_storage_model() {
    assert!(CREATE_STORE_TABLE_SQL.contains("value BLOB NOT NULL"));
    assert!(CREATE_STORE_TABLE_SQL.contains("codec TEXT NOT NULL"));
    assert!(CREATE_PREFIX_INDEX_SQL.contains("namespace, collection, key"));
    assert!(CREATE_EXPIRES_AT_INDEX_SQL.contains("expires_at"));
}

#[test]
fn driver_is_named_sqlite() {
    let driver = SqliteDriver::new("app.db");
    assert_eq!(driver.name(), "sqlite");
    assert!(!driver.capabilities().raw_sql);
}

#[test]
fn capabilities_report_implemented_storage_behaviour() {
    let capabilities = sqlite_capabilities();
    assert!(capabilities.transactions);
    assert!(capabilities.ttl);
    assert!(capabilities.prefix_scan);
    assert!(capabilities.batch_write);
    assert!(!capabilities.raw_sql);
}

#[test]
fn sqlite_constructor_returns_driver() {
    let driver = sqlite("app.db");
    assert_eq!(driver.name(), "sqlite");
}
