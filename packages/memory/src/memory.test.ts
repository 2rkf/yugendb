import { describe, expect, it } from "vitest";
import {
  JsonCodec,
  YugenDbError,
  createStore,
  createValueMetadata,
  normaliseCollectionName,
  normaliseKey,
  normaliseNamespace,
  type Batch,
  type StoredValue,
} from "@yugendb/core";
import { MemoryDriver, memory } from "./index";

const namespace = normaliseNamespace("tests");
const users = normaliseCollectionName("users");
const sessions = normaliseCollectionName("sessions");
const codec = new JsonCodec();

function stored(value: unknown, ttlMs?: number): StoredValue {
  return {
    bytes: codec.serialise(value),
    metadata: createValueMetadata(codec.name, ttlMs),
  };
}

describe("MemoryDriver", () => {
  it("reports its name and capabilities", () => {
    const driver = memory();

    expect(driver.name).toBe("memory");
    expect(driver.capabilities()).toMatchObject({
      transactions: false,
      ttl: true,
      prefixScan: true,
      batchWrite: true,
      rawSql: false,
      documentQuery: false,
    });
  });

  it("sets and gets values", async () => {
    const driver = memory();
    const key = normaliseKey("user_001");

    await driver.set(namespace, users, key, stored({ id: "user_001" }), {});

    const loaded = await driver.get(namespace, users, key, {});
    expect(codec.deserialise(loaded!.bytes)).toEqual({ id: "user_001" });
  });

  it("returns null for missing values", async () => {
    const driver = memory();
    await expect(driver.get(namespace, users, normaliseKey("missing"), {})).resolves.toBeNull();
  });

  it("deletes existing and missing values predictably", async () => {
    const driver = memory();
    const key = normaliseKey("user_001");

    await driver.set(namespace, users, key, stored({ id: "user_001" }), {});

    await expect(driver.delete(namespace, users, key, {})).resolves.toBe(true);
    await expect(driver.delete(namespace, users, key, {})).resolves.toBe(false);
    await expect(driver.delete(namespace, users, key, { mustExist: true })).rejects.toThrow(YugenDbError);
  });

  it("checks existence while respecting expiry", async () => {
    const driver = memory();
    const key = normaliseKey("session_001");

    await driver.set(namespace, sessions, key, stored({ id: "session_001" }, 0), {});

    await expect(driver.exists(namespace, sessions, key)).resolves.toBe(false);
  });

  it("isolates namespaces", async () => {
    const driver = memory();
    const key = normaliseKey("user_001");

    await driver.set(namespace, users, key, stored({ app: "one" }), {});

    await expect(driver.get(normaliseNamespace("other"), users, key, {})).resolves.toBeNull();
  });

  it("isolates collections", async () => {
    const driver = memory();
    const key = normaliseKey("shared_key");

    await driver.set(namespace, users, key, stored({ collection: "users" }), {});

    await expect(driver.get(namespace, sessions, key, {})).resolves.toBeNull();
  });

  it("scans prefixes in deterministic order", async () => {
    const driver = memory();

    await driver.set(namespace, users, normaliseKey("user_002"), stored({ id: "user_002" }), {});
    await driver.set(namespace, users, normaliseKey("admin_001"), stored({ id: "admin_001" }), {});
    await driver.set(namespace, users, normaliseKey("user_001"), stored({ id: "user_001" }), {});

    const rows = await driver.scanPrefix(namespace, users, { prefix: "user_" });

    expect(rows.map(([key]) => key)).toEqual(["user_001", "user_002"]);
  });

  it("applies batch set and delete operations", async () => {
    const driver = memory();
    const batch: Batch = {
      operations: [
        {
          kind: "set",
          namespace,
          collection: users,
          key: normaliseKey("user_001"),
          value: stored({ id: "user_001" }),
        },
        {
          kind: "set",
          namespace,
          collection: users,
          key: normaliseKey("user_002"),
          value: stored({ id: "user_002" }),
        },
        {
          kind: "delete",
          namespace,
          collection: users,
          key: normaliseKey("user_001"),
        },
      ],
    };

    await expect(driver.batch(batch)).resolves.toEqual({ appliedOperations: 3, deletedKeys: 1 });
    await expect(driver.get(namespace, users, normaliseKey("user_001"), {})).resolves.toBeNull();
    await expect(driver.get(namespace, users, normaliseKey("user_002"), {})).resolves.not.toBeNull();
  });

  it("hides expired values unless requested", async () => {
    const driver = memory();
    const key = normaliseKey("short_lived");

    await driver.set(namespace, users, key, stored({ id: "short_lived" }, 0), {});

    await expect(driver.get(namespace, users, key, {})).resolves.toBeNull();
    await expect(driver.get(namespace, users, key, { allowExpired: true })).resolves.not.toBeNull();
  });

  it("works through createStore and collection<T>", async () => {
    const db = await createStore({ driver: memory(), namespace: "tests" });
    const collection = db.collection<{ id: string }>("users");

    await collection.set("user_001", { id: "user_001" });

    await expect(collection.get("user_001")).resolves.toEqual({ id: "user_001" });
    await db.finalise();
  });

  it("can disable value cloning", () => {
    expect(new MemoryDriver({ cloneValues: false }).name).toBe("memory");
  });
});
