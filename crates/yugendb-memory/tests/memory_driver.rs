use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use yugendb_core::{
    Batch, BatchOperation, Codec, CodecExt, CollectionName, DeleteOptions, Driver, JsonCodec, Key,
    Namespace, ReadOptions, ScanOptions, Store, StoredValue, ValueMetadata, WriteOptions,
};
use yugendb_memory::{memory, MemoryDriver};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
}

fn user(id: &str) -> User {
    User {
        id: id.to_owned(),
        email: format!("{id}@example.com"),
    }
}

fn stored_text(value: &str) -> yugendb_core::Result<StoredValue> {
    let codec = JsonCodec;
    let bytes = codec.serialise(&value)?;
    let metadata = ValueMetadata::new(codec.name(), Utc::now(), None);
    Ok(StoredValue::new(bytes, metadata))
}

async fn store() -> yugendb_core::Result<Store> {
    Store::builder()
        .driver(memory())
        .namespace("memory_tests")?
        .connect()
        .await
}

#[tokio::test]
async fn reports_driver_name_and_capabilities() {
    let driver = MemoryDriver::new();

    assert_eq!(driver.name(), "memory");

    let capabilities = driver.capabilities();
    assert!(capabilities.ttl);
    assert!(capabilities.prefix_scan);
    assert!(capabilities.batch_write);
    assert!(!capabilities.transactions);
    assert!(!capabilities.raw_sql);
    assert!(!capabilities.document_query);
}

#[tokio::test]
async fn set_then_get_typed_value() -> yugendb_core::Result<()> {
    let store = store().await?;
    let users = store.collection::<User, _>("users")?;

    users.set("reader", &user("reader")).await?;

    let loaded = users.get("reader").await?;
    assert_eq!(loaded, Some(user("reader")));

    Ok(())
}

#[tokio::test]
async fn get_missing_returns_none() -> yugendb_core::Result<()> {
    let store = store().await?;
    let users = store.collection::<User, _>("users")?;

    assert_eq!(users.get("missing").await?, None);

    Ok(())
}

#[tokio::test]
async fn delete_and_exists_follow_missing_value_rules() -> yugendb_core::Result<()> {
    let store = store().await?;
    let users = store.collection::<User, _>("users")?;

    users.set("reader", &user("reader")).await?;
    assert!(users.exists("reader").await?);
    assert!(users.delete("reader").await?);
    assert!(!users.exists("reader").await?);
    assert!(!users.delete("reader").await?);

    Ok(())
}

#[tokio::test]
async fn delete_missing_can_require_existence() -> yugendb_core::Result<()> {
    let driver = memory();
    let namespace = Namespace::try_from("delete")?;
    let collection = CollectionName::try_from("users")?;
    let key = Key::try_from("missing")?;

    let error = driver
        .delete(
            &namespace,
            &collection,
            &key,
            &DeleteOptions { must_exist: true },
        )
        .await
        .expect_err("delete should fail when the key must exist");

    assert_eq!(error.code().as_str(), "NOT_FOUND");

    Ok(())
}

#[tokio::test]
async fn namespaces_are_isolated() -> yugendb_core::Result<()> {
    let driver = memory();

    let first = Store::builder()
        .driver(driver.clone())
        .namespace("first")?
        .connect()
        .await?;
    let second = Store::builder()
        .driver(driver)
        .namespace("second")?
        .connect()
        .await?;

    first.set("reader", &user("reader")).await?;

    let loaded: Option<User> = second.get("reader").await?;
    assert_eq!(loaded, None);

    Ok(())
}

#[tokio::test]
async fn collections_are_isolated() -> yugendb_core::Result<()> {
    let store = store().await?;
    let users = store.collection::<User, _>("users")?;
    let admins = store.collection::<User, _>("admins")?;

    users.set("reader", &user("reader")).await?;

    assert_eq!(admins.get("reader").await?, None);

    Ok(())
}

#[tokio::test]
async fn prefix_scan_is_sorted_and_limited() -> yugendb_core::Result<()> {
    let store = store().await?;
    let users = store.collection::<User, _>("users")?;

    users.set("user:2", &user("2")).await?;
    users.set("other:1", &user("other")).await?;
    users.set("user:1", &user("1")).await?;
    users.set("user:3", &user("3")).await?;

    let scanned = users.scan_prefix("user:").await?;
    let keys = scanned
        .into_iter()
        .map(|(key, _)| key.to_string())
        .collect::<Vec<_>>();

    assert_eq!(keys, vec!["user:1", "user:2", "user:3"]);

    let namespace = Namespace::try_from("memory_tests")?;
    let collection = CollectionName::try_from("users")?;
    let limited = store.capabilities();
    assert!(limited.prefix_scan);

    let driver = memory();
    driver
        .set(
            &namespace,
            &collection,
            &Key::try_from("user:2")?,
            stored_text("two")?,
            &WriteOptions::default(),
        )
        .await?;
    driver
        .set(
            &namespace,
            &collection,
            &Key::try_from("user:1")?,
            stored_text("one")?,
            &WriteOptions::default(),
        )
        .await?;

    let direct = driver
        .scan_prefix(
            &namespace,
            &collection,
            &ScanOptions {
                prefix: Some("user:".to_owned()),
                limit: Some(1),
                include_expired: false,
            },
        )
        .await?;

    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0].0.as_str(), "user:1");

    Ok(())
}

#[tokio::test]
async fn batch_sets_and_deletes_values() -> yugendb_core::Result<()> {
    let driver = memory();
    let namespace = Namespace::try_from("batch")?;
    let collection = CollectionName::try_from("users")?;
    let reader = Key::try_from("reader")?;
    let grace = Key::try_from("grace")?;

    let batch = Batch::new()
        .with_operation(BatchOperation::set(
            namespace.clone(),
            collection.clone(),
            reader.clone(),
            stored_text("Reader")?,
        ))
        .with_operation(BatchOperation::set(
            namespace.clone(),
            collection.clone(),
            grace.clone(),
            stored_text("Grace")?,
        ));

    let result = driver.batch(batch).await?;
    assert_eq!(result.set_count, 2);
    assert_eq!(result.delete_count, 0);
    assert!(driver.exists(&namespace, &collection, &reader).await?);

    let batch = Batch::new().with_operation(BatchOperation::delete(
        namespace.clone(),
        collection.clone(),
        reader.clone(),
    ));

    let result = driver.batch(batch).await?;
    assert_eq!(result.set_count, 0);
    assert_eq!(result.delete_count, 1);
    assert!(!driver.exists(&namespace, &collection, &reader).await?);
    assert!(driver.exists(&namespace, &collection, &grace).await?);

    Ok(())
}

#[tokio::test]
async fn ttl_expiry_hides_values() -> yugendb_core::Result<()> {
    let store = store().await?;
    let collection = CollectionName::try_from("users")?;

    store
        .set_in_collection_with_options(
            &collection,
            "reader",
            &user("reader"),
            WriteOptions {
                ttl: Some(Duration::from_millis(1)),
                overwrite: true,
            },
        )
        .await?;

    std::thread::sleep(Duration::from_millis(10));

    let users = store.collection::<User, _>("users")?;
    assert_eq!(users.get("reader").await?, None);
    assert!(!users.exists("reader").await?);

    Ok(())
}

#[tokio::test]
async fn direct_get_can_allow_expired_values() -> yugendb_core::Result<()> {
    let driver = memory();
    let namespace = Namespace::try_from("ttl")?;
    let collection = CollectionName::try_from("users")?;
    let key = Key::try_from("reader")?;
    let now = Utc::now();
    let metadata = ValueMetadata::new("json", now, Some(now - chrono::Duration::milliseconds(1)));
    let value = StoredValue::new(JsonCodec.serialise(&user("reader"))?, metadata);

    driver
        .set(
            &namespace,
            &collection,
            &key,
            value,
            &WriteOptions::default(),
        )
        .await?;

    assert!(driver
        .get(&namespace, &collection, &key, &ReadOptions::default())
        .await?
        .is_none());
    assert!(driver
        .get(
            &namespace,
            &collection,
            &key,
            &ReadOptions {
                allow_expired: true,
            },
        )
        .await?
        .is_some());

    Ok(())
}
