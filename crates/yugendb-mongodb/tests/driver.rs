use yugendb_core::Driver;
use yugendb_mongodb::{
    identity_index_fields, mongodb, mongodb_capabilities, MongoDbDriver,
    STORAGE_COLLECTION_NAME, STORED_DOCUMENT_FIELDS,
};

#[test]
fn document_mapping_is_coherent() {
    assert_eq!(STORAGE_COLLECTION_NAME, "yugendb_store");
    assert_eq!(identity_index_fields(), ["namespace", "collection", "key"]);
    assert!(STORED_DOCUMENT_FIELDS.contains(&"value"));
    assert!(STORED_DOCUMENT_FIELDS.contains(&"expiresAt"));
}

#[test]
fn driver_name_options_and_capabilities_are_coherent() {
    let driver = MongoDbDriver::new("mongodb://localhost:27017");
    assert_eq!(driver.name(), "mongodb");
    assert_eq!(driver.options().database, "yugendb");
    assert!(!driver.capabilities().document_query);
    assert!(driver.capabilities().prefix_scan);
    assert!(!mongodb_capabilities().document_query);
}

#[test]
fn constructor_returns_driver() {
    let driver = mongodb("mongodb://localhost:27017");
    assert_eq!(driver.name(), "mongodb");
}
