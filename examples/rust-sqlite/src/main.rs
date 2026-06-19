//! Simple SQLite example for yugendb.

use serde::{Deserialize, Serialize};
use yugendb::drivers::sqlite::sqlite;
use yugendb::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let store = Store::builder()
        .driver(sqlite(":memory:"))
        .namespace("sqlite_example")?
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
