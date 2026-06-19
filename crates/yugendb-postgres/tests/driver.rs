use yugendb_core::Driver;
use yugendb_postgres::{
    postgres, postgres_capabilities, schema_statements, PostgresDriver,
    CREATE_EXPIRES_AT_INDEX_SQL, CREATE_PREFIX_INDEX_SQL, CREATE_STORE_TABLE_SQL, STORE_TABLE_NAME,
};

#[test]
fn schema_mentions_storage_table_and_indexes() {
    assert_eq!(STORE_TABLE_NAME, "yugendb_store");
    assert!(CREATE_STORE_TABLE_SQL.contains("CREATE TABLE"));
    assert!(CREATE_STORE_TABLE_SQL.contains("yugendb_store"));
    assert!(CREATE_PREFIX_INDEX_SQL.contains("idx_yugendb_store_prefix"));
    assert!(CREATE_EXPIRES_AT_INDEX_SQL.contains("idx_yugendb_store_expires_at"));
    assert_eq!(schema_statements().len(), 3);
}

#[test]
fn driver_name_options_and_capabilities_are_coherent() {
    let driver = PostgresDriver::new("backend://localhost/yugendb");
    assert_eq!(driver.name(), "postgres");
    assert_eq!(
        driver.options().connection_string,
        "backend://localhost/yugendb"
    );
    assert!(driver.capabilities().prefix_scan);
    assert!(postgres_capabilities().prefix_scan);
    assert!(postgres_capabilities().batch_write);
}

#[test]
fn constructor_returns_driver() {
    let driver = postgres("backend://localhost/yugendb");
    assert_eq!(driver.name(), "postgres");
}
