//! Complete Rust memory driver example for yugendb.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use yugendb::drivers::memory;
use yugendb::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Session {
    id: String,
    user_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let driver = memory();
    let store = Store::builder()
        .driver(driver.clone())
        .namespace("rust_memory_example")?
        .connect()
        .await?;

    let capabilities = store.capabilities();
    println!(
        "Memory capabilities: ttl={}, prefix_scan={}, batch_write={}",
        capabilities.ttl, capabilities.prefix_scan, capabilities.batch_write
    );

    let users = store.collection::<User, _>("users")?;

    users
        .set(
            "user_001",
            &User {
                id: "user_001".to_owned(),
                email: "reader@example.com".to_owned(),
            },
        )
        .await?;

    users
        .set(
            "user_002",
            &User {
                id: "user_002".to_owned(),
                email: "grace@example.com".to_owned(),
            },
        )
        .await?;

    let loaded: Option<User> = users.get("user_001").await?;
    println!("Loaded user: {loaded:?}");

    let grace_exists = users.exists("user_002").await?;
    println!("Grace exists: {grace_exists}");

    let scanned = users.scan_prefix("user_").await?;
    println!("Prefix scan matched {} users", scanned.len());

    let deleted = users.delete("user_001").await?;
    println!("Reader deleted: {deleted}");

    let missing_after_delete: Option<User> = users.get("user_001").await?;
    println!("Reader after delete: {missing_after_delete:?}");

    show_structured_error(&store)?;
    run_batch_example(&driver).await?;

    store.finalise().await?;

    Ok(())
}

fn show_structured_error(store: &Store) -> Result<()> {
    match store.collection::<User, _>("") {
        Ok(_) => println!("Unexpectedly created an empty collection name"),
        Err(error) => {
            println!("Expected structured error code: {}", error.code().as_str());
        }
    }

    Ok(())
}

async fn run_batch_example(driver: &impl Driver) -> Result<()> {
    let namespace = Namespace::try_from("rust_memory_example")?;
    let collection = CollectionName::try_from("sessions")?;
    let key = Key::try_from("session_user_001")?;
    let codec = JsonCodec;
    let now = Utc::now();

    let session = Session {
        id: "session_user_001".to_owned(),
        user_id: "user_001".to_owned(),
    };

    let stored = StoredValue::new(
        codec.serialise(&session)?,
        ValueMetadata::new(codec.name(), now, None),
    );

    let set_batch = Batch::new().with_operation(BatchOperation::set(
        namespace.clone(),
        collection.clone(),
        key.clone(),
        stored,
    ));
    let set_result = driver.batch(set_batch).await?;
    println!("Batch set operations: {}", set_result.set_count);

    let delete_batch =
        Batch::new().with_operation(BatchOperation::delete(namespace, collection, key));
    let delete_result = driver.batch(delete_batch).await?;
    println!("Batch delete operations: {}", delete_result.delete_count);

    Ok(())
}
