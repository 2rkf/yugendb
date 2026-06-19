use yugendb_core::Driver;
use yugendb_redis::{
    compose_collection_prefix, compose_storage_key, redis, redis_capabilities, RedisDriver,
    DEFAULT_KEY_PREFIX,
};

#[test]
fn keyspace_mapping_is_deterministic() {
    assert_eq!(DEFAULT_KEY_PREFIX, "yugendb");
    assert_eq!(
        compose_storage_key("yugendb", "app", "users", "001"),
        "yugendb:app:users:001"
    );
    assert_eq!(
        compose_collection_prefix("yugendb", "app", "users"),
        "yugendb:app:users:"
    );
}

#[test]
fn driver_name_options_and_capabilities_are_coherent() {
    let driver = RedisDriver::new("redis://localhost:6379").expect("driver");
    assert_eq!(driver.name(), "redis");
    assert_eq!(driver.options().key_prefix, "yugendb");
    assert!(driver.capabilities().prefix_scan);
    assert_eq!(redis_capabilities().prefix_scan, true);
}

#[test]
fn constructor_returns_driver() {
    let driver = redis("redis://localhost:6379").expect("driver");
    assert_eq!(driver.name(), "redis");
}
