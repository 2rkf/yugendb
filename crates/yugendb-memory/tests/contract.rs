use std::{fs, path::PathBuf, time::Duration};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use yugendb_core::{
    Batch, BatchOperation, Codec, CodecExt, CollectionName, DeleteOptions, Driver, ErrorCode,
    JsonCodec, Key, Namespace, ReadOptions, ScanOptions, Store, StoredValue, ValueMetadata,
    WriteOptions,
};
use yugendb_memory::memory;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ContractUser {
    id: String,
    email: String,
}

fn contract_user(id: &str) -> ContractUser {
    ContractUser {
        id: id.to_owned(),
        email: format!("{id}@example.com"),
    }
}

fn stored_json<T: Serialize>(value: &T) -> yugendb_core::Result<StoredValue> {
    let codec = JsonCodec;
    let bytes = codec.serialise(value)?;
    let metadata = ValueMetadata::new(codec.name(), Utc::now(), None);
    Ok(StoredValue::new(bytes, metadata))
}

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compatibility/fixtures")
        .join(name);
    let text = fs::read_to_string(path).expect("compatibility fixture should be readable");
    serde_json::from_str(&text).expect("compatibility fixture should be valid JSON")
}

async fn contract_store(namespace: &str) -> yugendb_core::Result<Store> {
    Store::builder()
        .driver(memory())
        .namespace(namespace)?
        .connect()
        .await
}

#[test]
fn shared_fixtures_are_valid_json() {
    for name in [
        "basic.json",
        "errors.json",
        "capabilities.json",
        "namespaces.json",
        "collections.json",
        "prefix-scan.json",
        "batch.json",
        "ttl.json",
        "serialisation.json",
    ] {
        let value = fixture(name);
        assert_eq!(value["version"], 1);
    }
}

#[tokio::test]
async fn memory_driver_matches_basic_store_contract() -> yugendb_core::Result<()> {
    let store = contract_store("contract_basic").await?;
    let users = store.collection::<ContractUser, _>("users")?;

    users.set("user_001", &contract_user("user_001")).await?;
    assert_eq!(
        users.get("user_001").await?,
        Some(contract_user("user_001"))
    );

    users
        .set("user_001", &contract_user("user_001_new"))
        .await?;
    assert_eq!(
        users.get("user_001").await?,
        Some(contract_user("user_001_new"))
    );

    assert_eq!(users.get("missing_user").await?, None);
    assert!(users.exists("user_001").await?);
    assert!(!users.exists("missing_user").await?);
    assert!(users.delete("user_001").await?);
    assert!(!users.delete("user_001").await?);

    Ok(())
}

#[tokio::test]
async fn memory_driver_matches_namespace_and_collection_contract() -> yugendb_core::Result<()> {
    let driver = memory();
    let first = Store::builder()
        .driver(driver.clone())
        .namespace("tenant_a")?
        .connect()
        .await?;
    let second = Store::builder()
        .driver(driver.clone())
        .namespace("tenant_b")?
        .connect()
        .await?;

    first.set("shared", &contract_user("tenant_a")).await?;
    assert_eq!(second.get::<ContractUser, _>("shared").await?, None);

    let users = first.collection::<ContractUser, _>("users")?;
    let admins = first.collection::<ContractUser, _>("admins")?;
    users.set("shared", &contract_user("user")).await?;
    assert_eq!(admins.get("shared").await?, None);

    Ok(())
}

#[tokio::test]
async fn memory_driver_matches_prefix_scan_contract() -> yugendb_core::Result<()> {
    let store = contract_store("contract_scan").await?;
    let users = store.collection::<ContractUser, _>("users")?;

    users.set("user_002", &contract_user("user_002")).await?;
    users.set("admin_001", &contract_user("admin_001")).await?;
    users.set("user_001", &contract_user("user_001")).await?;

    let keys = users
        .scan_prefix("user_")
        .await?
        .into_iter()
        .map(|(key, _)| key.to_string())
        .collect::<Vec<_>>();

    assert_eq!(keys, vec!["user_001", "user_002"]);

    let driver = memory();
    let namespace = Namespace::try_from("contract_scan_limit")?;
    let collection = CollectionName::try_from("users")?;
    for key in ["user_003", "user_001", "user_002"] {
        driver
            .set(
                &namespace,
                &collection,
                &Key::try_from(key)?,
                stored_json(&contract_user(key))?,
                &WriteOptions::default(),
            )
            .await?;
    }

    let limited = driver
        .scan_prefix(
            &namespace,
            &collection,
            &ScanOptions {
                prefix: Some("user_".to_owned()),
                limit: Some(2),
                include_expired: false,
            },
        )
        .await?;
    assert_eq!(
        limited
            .iter()
            .map(|(key, _)| key.to_string())
            .collect::<Vec<_>>(),
        vec!["user_001", "user_002"]
    );

    Ok(())
}

#[tokio::test]
async fn memory_driver_matches_batch_contract() -> yugendb_core::Result<()> {
    let driver = memory();
    let namespace = Namespace::try_from("contract_batch")?;
    let collection = CollectionName::try_from("users")?;
    let user_001 = Key::try_from("user_001")?;
    let user_002 = Key::try_from("user_002")?;

    let batch = Batch::new()
        .with_operation(BatchOperation::set(
            namespace.clone(),
            collection.clone(),
            user_001.clone(),
            stored_json(&contract_user("user_001"))?,
        ))
        .with_operation(BatchOperation::set(
            namespace.clone(),
            collection.clone(),
            user_002.clone(),
            stored_json(&contract_user("user_002"))?,
        ))
        .with_operation(BatchOperation::delete(
            namespace.clone(),
            collection.clone(),
            user_001.clone(),
        ));

    let result = driver.batch(batch).await?;
    assert_eq!(result.set_count, 2);
    assert_eq!(result.delete_count, 1);
    assert!(!driver.exists(&namespace, &collection, &user_001).await?);
    assert!(driver.exists(&namespace, &collection, &user_002).await?);

    Ok(())
}

#[tokio::test]
async fn memory_driver_matches_ttl_contract() -> yugendb_core::Result<()> {
    let store = contract_store("contract_ttl").await?;
    let collection = CollectionName::try_from("sessions")?;

    store
        .set_in_collection_with_options(
            &collection,
            "session_old",
            &contract_user("session_old"),
            WriteOptions {
                ttl: Some(Duration::from_millis(0)),
                overwrite: true,
            },
        )
        .await?;
    store
        .set_in_collection_with_options(
            &collection,
            "session_new",
            &contract_user("session_new"),
            WriteOptions {
                ttl: Some(Duration::from_secs(60)),
                overwrite: true,
            },
        )
        .await?;

    let sessions = store.collection::<ContractUser, _>("sessions")?;
    assert_eq!(sessions.get("session_old").await?, None);
    assert!(!sessions.exists("session_old").await?);
    let keys = sessions
        .scan_prefix("session_")
        .await?
        .into_iter()
        .map(|(key, _)| key.to_string())
        .collect::<Vec<_>>();
    assert_eq!(keys, vec!["session_new"]);

    let driver = memory();
    let namespace = Namespace::try_from("contract_ttl_direct")?;
    let collection = CollectionName::try_from("sessions")?;
    let key = Key::try_from("expired")?;
    let now = Utc::now();
    let expired = StoredValue::new(
        JsonCodec.serialise(&contract_user("expired"))?,
        ValueMetadata::new("json", now, Some(now - chrono::Duration::milliseconds(1))),
    );
    driver
        .set(
            &namespace,
            &collection,
            &key,
            expired,
            &WriteOptions::default(),
        )
        .await?;
    assert!(driver
        .get(
            &namespace,
            &collection,
            &key,
            &ReadOptions {
                allow_expired: true
            },
        )
        .await?
        .is_some());

    Ok(())
}

#[tokio::test]
async fn memory_driver_matches_error_and_capability_contract() -> yugendb_core::Result<()> {
    let driver = memory();
    let capabilities = driver.capabilities();

    assert!(!capabilities.transactions);
    assert!(capabilities.ttl);
    assert!(capabilities.prefix_scan);
    assert!(capabilities.batch_write);
    assert!(!capabilities.raw_sql);
    assert!(!capabilities.document_query);

    assert_eq!(Key::try_from("").unwrap_err().code(), ErrorCode::InvalidKey);
    assert_eq!(
        Namespace::try_from("").unwrap_err().code(),
        ErrorCode::InvalidNamespace
    );
    assert_eq!(
        CollectionName::try_from("").unwrap_err().code(),
        ErrorCode::InvalidCollection
    );

    let namespace = Namespace::try_from("contract_errors")?;
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
        .expect_err("must_exist delete should fail for missing values");
    assert_eq!(error.code(), ErrorCode::NotFound);

    Ok(())
}
