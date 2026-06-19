import { describe, expect, it } from "vitest";
import { createStore } from "@yugendb/core";
import { mongodb } from "./index.js";

type User = { id: string; email: string };

describe("MongoDB integration", () => {
  it("runs storage checks only when YUGENDB_MONGODB_URL is set", async () => {
    const url = process.env.YUGENDB_MONGODB_URL;
    if (url === undefined || url.length === 0) {
      expect(true).toBe(true);
      return;
    }

    const namespace = `integration_${Date.now()}_${Math.random().toString(16).slice(2)}`;
    const db = await createStore({ driver: mongodb(url), namespace });
    const users = db.collection<User>("users");

    await users.set("user_001", { id: "user_001", email: "reader@example.com" });
    expect(await users.get("user_001")).toEqual({ id: "user_001", email: "reader@example.com" });
    expect(await users.get("missing")).toBeNull();
    expect(await users.exists("user_001")).toBe(true);

    await users.set("user_002", { id: "user_002", email: "grace@example.com" });
    const scanned = await users.scanPrefix("user_");
    expect(scanned.map(([key]) => key)).toContain("user_001");

    expect(await users.delete("user_001")).toBe(true);
    expect(await users.delete("user_001")).toBe(false);
    await db.finalise();
  });
});
