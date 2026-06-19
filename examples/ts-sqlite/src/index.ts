import { createStore } from "@yugendb/core";
import { sqlite } from "@yugendb/sqlite";

type User = {
  readonly id: string;
  readonly email: string;
};

const db = await createStore({
  driver: sqlite(":memory:"),
  namespace: "sqlite_example",
});

try {
  const users = db.collection<User>("users");

  await users.set("user_001", {
    id: "user_001",
    email: "reader@example.com",
  });

  const loaded = await users.get("user_001");
  console.log("Loaded user:", loaded);
} finally {
  await db.finalise();
}
