//! Minimal Rust example for yugendb.

use serde::{Deserialize, Serialize};
use yugendb::drivers::memory;
use yugendb::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let store = Store::builder()
        .driver(memory())
        .namespace("example_app")?
        .connect()
        .await?;

    let user = User {
        id: "user_001".to_owned(),
        email: "reader@example.com".to_owned(),
    };

    store.set("users:user_001", &user).await?;

    let loaded: Option<User> = store.get("users:user_001").await?;
    println!("Loaded user: {loaded:?}");

    store.finalise().await?;

    Ok(())
}
