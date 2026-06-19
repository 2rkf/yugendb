use chrono::Utc;
use yugendb_core::{
    CollectionName, DeleteOptions, Driver, Key, Namespace, ReadOptions, ScanOptions, StoredValue,
    ValueBytes, ValueMetadata, WriteOptions,
};
use yugendb_postgres::postgres;

#[tokio::test]
async fn storage_contract_when_environment_is_configured() {
    let Ok(url) = std::env::var("YUGENDB_POSTGRES_URL") else {
        return;
    };

    let driver = postgres(url);
    driver.initialise().await.expect("initialise driver");

    let namespace = Namespace::new(format!(
        "integration_{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ))
    .expect("namespace");
    let collection = CollectionName::new("users").expect("collection");
    let key = Key::new("user_001").expect("key");
    let metadata = ValueMetadata::new("json", Utc::now(), None);
    let stored = StoredValue::new(ValueBytes::from(br#"{"id":"user_001"}"#.to_vec()), metadata);

    driver
        .set(
            &namespace,
            &collection,
            &key,
            stored.clone(),
            &WriteOptions::default(),
        )
        .await
        .expect("set");
    let loaded = driver
        .get(&namespace, &collection, &key, &ReadOptions::default())
        .await
        .expect("get")
        .expect("value");
    assert_eq!(loaded.bytes.as_slice(), stored.bytes.as_slice());
    assert!(driver
        .exists(&namespace, &collection, &key)
        .await
        .expect("exists"));

    let scanned = driver
        .scan_prefix(
            &namespace,
            &collection,
            &ScanOptions {
                prefix: Some("user_".to_owned()),
                limit: None,
                include_expired: false,
            },
        )
        .await
        .expect("scan");
    assert!(scanned
        .iter()
        .any(|(stored_key, _)| stored_key.as_str() == "user_001"));

    assert!(driver
        .delete(&namespace, &collection, &key, &DeleteOptions::default())
        .await
        .expect("delete"));
    assert!(!driver
        .delete(&namespace, &collection, &key, &DeleteOptions::default())
        .await
        .expect("delete missing"));
    driver.finalise().await.expect("finalise");
}
