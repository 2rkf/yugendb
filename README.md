# YugenDB

**YugenDB** is a typed, driver-based key-value and document storage layer for Rust and TypeScript.

## Install

Rust:

```toml
[dependencies]
yugendb = { version = "0.1", features = ["sqlite"] }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

TypeScript:

```bash
pnpm add @yugendb/core @yugendb/sqlite
```

## Rust Quickstart

```rust
use serde::{Deserialize, Serialize};
use yugendb::drivers::sqlite::sqlite;
use yugendb::prelude::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Store::builder()
        .driver(sqlite(":memory:"))
        .namespace("example_app")?
        .connect()
        .await?;

    let users = db.collection::<User, _>("users")?;
    users
        .set(
            "user_001",
            &User {
                id: "user_001".into(),
                email: "reader@example.com".into(),
            },
        )
        .await?;

    let loaded: Option<User> = users.get("user_001").await?;
    println!("Loaded user: {loaded:?}");

    db.finalise().await?;
    Ok(())
}
```

## TypeScript Quickstart

```ts
import { createStore } from "@yugendb/core";
import { sqlite } from "@yugendb/sqlite";

type User = {
  id: string;
  email: string;
};

const db = await createStore({
  driver: sqlite(":memory:"),
  namespace: "example_app",
});

const users = db.collection<User>("users");
await users.set("user_001", {
  id: "user_001",
  email: "reader@example.com",
});

const loaded = await users.get("user_001");
console.log("Loaded user:", loaded);

await db.finalise();
```

Rust returns `Option<T>` for reads. TypeScript returns `T | null`.

## Drivers

| Driver | Rust crate feature | TypeScript package | Notes |
|---|---|---|---|
| Memory | `memory` | `@yugendb/memory` | In-process storage. |
| SQLite | `sqlite` | `@yugendb/sqlite` | Local SQLite storage. |
| PostgreSQL | `postgres` | `@yugendb/postgres` | SQL-backed external service. |
| MySQL | `mysql` | `@yugendb/mysql` | SQL-backed external service. |
| MongoDB | `mongodb` | `@yugendb/mongodb` | Document-backed external service. |
| Redis | `redis` | `@yugendb/redis` | Key-value external service. |

External-service drivers need a live database connection string at runtime.

YugenDB is released under the MIT Licence. See [LICENSE.md](./LICENSE.md).
