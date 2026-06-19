
import { createStore } from "@yugendb/core";
import { memory } from "@yugendb/memory";

type User = {
  readonly id: string;
  readonly email: string;
};

const db = await createStore({
  driver: memory(),
  namespace: "example_app",
});

try {
  const user = {
    id: "user_001",
    email: "reader@example.com",
  } satisfies User;

  await db.set("users:user_001", user);

  const loaded = await db.get<User>("users:user_001");

  console.log("Loaded user:", loaded);
} finally {
  await db.finalise();
}
