import { createStore } from "@yugendb/core";
import { mysql } from "@yugendb/mysql";

type User = {
  readonly id: string;
  readonly email: string;
};

const connectionString = process.env.YUGENDB_MYSQL_URL;
if (connectionString === undefined || connectionString.length === 0) {
  console.log("Set YUGENDB_MYSQL_URL to run this example.");
  process.exit(0);
}

const db = await createStore({
  driver: mysql(connectionString),
  namespace: "mysql_example",
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
