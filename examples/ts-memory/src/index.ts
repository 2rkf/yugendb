
import { createStore, isYugenDbError } from "@yugendb/core";
import { memory } from "@yugendb/memory";

type User = {
  readonly id: string;
  readonly email: string;
  readonly role: "admin" | "member";
};

type Session = {
  readonly id: string;
  readonly userId: string;
};

const db = await createStore({
  driver: memory(),
  namespace: "memory_example",
});

try {
  const capabilities = db.capabilities();
  console.log("Supports prefix scan:", capabilities.prefixScan);
  console.log("Supports TTL:", capabilities.ttl);
  console.log("Supports batch write:", capabilities.batchWrite);

  const users = db.collection<User>("users");
  const sessions = db.collection<Session>("sessions");

  await users.set("user_001", {
    id: "user_001",
    email: "reader@example.com",
    role: "admin",
  });

  await users.set("user_002", {
    id: "user_002",
    email: "grace@example.com",
    role: "member",
  });

  await sessions.set(
    "session_001",
    {
      id: "session_001",
      userId: "user_001",
    },
    { ttlMs: 60_000 },
  );

  const loaded = await users.get("user_001");
  console.log("Loaded user:", loaded);

  const missing = await users.get("missing_user");
  console.log("Missing user is null:", missing === null);

  const userExists = await users.exists("user_002");
  console.log("Does user_002 exist?", userExists);

  const matches = await users.scanPrefix("user_");
  console.log("Prefix scan matches:", matches);

  const sessionBeforeExpiry = await sessions.get("session_001");
  console.log("Session before expiry:", sessionBeforeExpiry);

  const deleted = await users.delete("user_002");
  console.log("Deleted user_002?", deleted);

  const afterDelete = await users.exists("user_002");
  console.log("Does user_002 exist after delete?", afterDelete);

  try {
    db.collection<User>("");
  } catch (error) {
    if (isYugenDbError(error)) {
      console.log("Expected collection error code:", error.code);
    } else {
      throw error;
    }
  }

  try {
    await users.delete("missing_user", { mustExist: true });
  } catch (error) {
    if (isYugenDbError(error)) {
      console.log("Expected delete error code:", error.code);
    } else {
      throw error;
    }
  }

  // Batch examples will be expanded once the public batch helper API is finalised.
} finally {
  await db.finalise();
}
