//! Simple Redis example for yugendb.

use serde::{Deserialize, Serialize};
use std::env;
use yugendb::drivers::redis::redis;
use yugendb::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Ok(connection_string) = env::var("YUGENDB_REDIS_URL") else {
        println!("Set YUGENDB_REDIS_URL to run this example.");
        return Ok(());
    };

    let store = Store::builder()
        .driver(redis(connection_string)?)
        .namespace("redis_example")?
        .connect()
        .await?;

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

    let loaded: Option<User> = users.get("user_001").await?;
    println!("Loaded user: {loaded:?}");

    store.finalise().await?;
    Ok(())
}
