use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration as ChronoDuration, Utc};
use yugendb_core::{
    Batch, BatchOperation, CollectionName, DeleteOptions, Driver, Key, Namespace, ReadOptions,
    ScanOptions, StoredValue, ValueBytes, ValueMetadata,
};
use yugendb_sqlite::sqlite;

fn temp_db_path(name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("yugendb-{name}-{nanos}.sqlite"))
        .to_string_lossy()
        .into_owned()
}

fn stored(bytes: &[u8]) -> StoredValue {
    StoredValue::new(
        ValueBytes::from(bytes.to_vec()),
        ValueMetadata::new("json", Utc::now(), None),
    )
}

#[tokio::test]
async fn sqlite_set_get_delete_and_exists() {
    let driver = sqlite(temp_db_path("basic"));
    driver.initialise().await.unwrap();

    let namespace = Namespace::try_from("app").unwrap();
    let collection = CollectionName::try_from("users").unwrap();
    let key = Key::try_from("user_001").unwrap();

    assert!(!driver.exists(&namespace, &collection, &key).await.unwrap());
    driver
        .set(
            &namespace,
            &collection,
            &key,
            stored(br#"{"name":"Reader"}"#),
            &Default::default(),
        )
        .await
        .unwrap();

    assert!(driver.exists(&namespace, &collection, &key).await.unwrap());
    let loaded = driver
        .get(&namespace, &collection, &key, &ReadOptions::default())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded.bytes.as_slice(), br#"{"name":"Reader"}"#);

    assert!(driver
        .delete(&namespace, &collection, &key, &DeleteOptions::default())
        .await
        .unwrap());
    assert!(driver
        .get(&namespace, &collection, &key, &ReadOptions::default())
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn sqlite_scan_prefix_and_batch_are_deterministic() {
    let driver = sqlite(temp_db_path("scan"));
    driver.initialise().await.unwrap();

    let namespace = Namespace::try_from("app").unwrap();
    let collection = CollectionName::try_from("sessions").unwrap();
    let user_a = Key::try_from("user_a").unwrap();
    let user_b = Key::try_from("user_b").unwrap();
    let other = Key::try_from("other").unwrap();

    let mut batch = Batch::new();
    batch.push(BatchOperation::set(
        namespace.clone(),
        collection.clone(),
        user_b.clone(),
        stored(b"b"),
    ));
    batch.push(BatchOperation::set(
        namespace.clone(),
        collection.clone(),
        user_a.clone(),
        stored(b"a"),
    ));
    batch.push(BatchOperation::set(
        namespace.clone(),
        collection.clone(),
        other,
        stored(b"other"),
    ));

    let result = driver.batch(batch).await.unwrap();
    assert_eq!(result.set_count, 3);

    let matches = driver
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
        .unwrap();

    let keys: Vec<_> = matches
        .into_iter()
        .map(|(key, _)| key.to_string())
        .collect();
    assert_eq!(keys, vec!["user_a", "user_b"]);
}

#[tokio::test]
async fn sqlite_hides_expired_values_by_default() {
    let driver = sqlite(temp_db_path("ttl"));
    driver.initialise().await.unwrap();

    let namespace = Namespace::try_from("app").unwrap();
    let collection = CollectionName::try_from("sessions").unwrap();
    let key = Key::try_from("session_001").unwrap();
    let expired = StoredValue::new(
        ValueBytes::from(b"expired".to_vec()),
        ValueMetadata::new(
            "json",
            Utc::now(),
            Some(Utc::now() - ChronoDuration::seconds(1)),
        ),
    );

    driver
        .set(&namespace, &collection, &key, expired, &Default::default())
        .await
        .unwrap();

    assert!(driver
        .get(&namespace, &collection, &key, &ReadOptions::default())
        .await
        .unwrap()
        .is_none());
    assert!(!driver.exists(&namespace, &collection, &key).await.unwrap());

    let loaded = driver
        .get(
            &namespace,
            &collection,
            &key,
            &ReadOptions {
                allow_expired: true,
            },
        )
        .await
        .unwrap();
    assert!(loaded.is_some());
}
