import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  JsonCodec,
  YugenDbError,
  createStore,
  createValueMetadata,
  isYugenDbError,
  normaliseCollectionName,
  normaliseKey,
  normaliseNamespace,
  type Batch,
  type StoredValue,
} from "@yugendb/core";
import { memory } from "./index";

const currentDirectory = dirname(fileURLToPath(import.meta.url));
const fixtureDirectory = resolve(currentDirectory, "../../../tests/compatibility/fixtures");
const codec = new JsonCodec();

function fixture(name: string): unknown {
  return JSON.parse(readFileSync(resolve(fixtureDirectory, name), "utf8"));
}

function stored(value: unknown, ttlMs?: number): StoredValue {
  return {
    bytes: codec.serialise(value),
    metadata: createValueMetadata(codec.name, ttlMs),
  };
}

type User = {
  readonly id: string;
  readonly email: string;
};

function user(id: string): User {
  return { id, email: `${id}@example.com` };
}

describe("TypeScript memory driver contract", () => {
  it("can read all shared compatibility fixtures", () => {
    for (const name of [
      "basic.json",
      "errors.json",
      "capabilities.json",
      "namespaces.json",
      "collections.json",
      "prefix-scan.json",
      "batch.json",
      "ttl.json",
      "serialisation.json",
    ]) {
      expect(fixture(name)).toMatchObject({ version: 1 });
    }
  });

  it("matches the basic store contract", async () => {
    const db = await createStore({ driver: memory(), namespace: "contract_basic" });
    const users = db.collection<User>("users");

    await users.set("user_001", user("user_001"));
    await expect(users.get("user_001")).resolves.toEqual(user("user_001"));

    await users.set("user_001", user("user_001_new"));
    await expect(users.get("user_001")).resolves.toEqual(user("user_001_new"));

    await expect(users.get("missing_user")).resolves.toBeNull();
    await expect(users.exists("user_001")).resolves.toBe(true);
    await expect(users.exists("missing_user")).resolves.toBe(false);
    await expect(users.delete("user_001")).resolves.toBe(true);
    await expect(users.delete("user_001")).resolves.toBe(false);
  });

  it("matches namespace and collection isolation contract", async () => {
    const driver = memory();
    const first = await createStore({ driver, namespace: "tenant_a" });
    const second = await createStore({ driver, namespace: "tenant_b" });

    await first.set("shared", user("tenant_a"));
    await expect(second.get<User>("shared")).resolves.toBeNull();

    const users = first.collection<User>("users");
    const admins = first.collection<User>("admins");
    await users.set("shared", user("user"));
    await expect(admins.get("shared")).resolves.toBeNull();
  });

  it("matches prefix scan contract", async () => {
    const db = await createStore({ driver: memory(), namespace: "contract_scan" });
    const users = db.collection<User>("users");

    await users.set("user_002", user("user_002"));
    await users.set("admin_001", user("admin_001"));
    await users.set("user_001", user("user_001"));

    await expect(users.scanPrefix("user_")).resolves.toEqual([
      [normaliseKey("user_001"), user("user_001")],
      [normaliseKey("user_002"), user("user_002")],
    ]);

    const driver = memory();
    const namespace = normaliseNamespace("contract_scan_limit");
    const collection = normaliseCollectionName("users");
    for (const key of ["user_003", "user_001", "user_002"]) {
      await driver.set(namespace, collection, normaliseKey(key), stored(user(key)), {});
    }

    const limited = await driver.scanPrefix(namespace, collection, {
      prefix: "user_",
      limit: 2,
    });
    expect(limited.map(([key]) => key)).toEqual(["user_001", "user_002"]);
  });

  it("matches batch operation contract", async () => {
    const driver = memory();
    const namespace = normaliseNamespace("contract_batch");
    const collection = normaliseCollectionName("users");
    const batch: Batch = {
      operations: [
        {
          kind: "set",
          namespace,
          collection,
          key: normaliseKey("user_001"),
          value: stored(user("user_001")),
        },
        {
          kind: "set",
          namespace,
          collection,
          key: normaliseKey("user_002"),
          value: stored(user("user_002")),
        },
        {
          kind: "delete",
          namespace,
          collection,
          key: normaliseKey("user_001"),
        },
      ],
    };

    await expect(driver.batch(batch)).resolves.toEqual({
      appliedOperations: 3,
      deletedKeys: 1,
    });
    await expect(driver.get(namespace, collection, normaliseKey("user_001"), {})).resolves.toBeNull();
    await expect(driver.exists(namespace, collection, normaliseKey("user_002"))).resolves.toBe(true);
  });

  it("matches TTL contract", async () => {
    const db = await createStore({ driver: memory(), namespace: "contract_ttl" });
    const sessions = db.collection<User>("sessions");

    await sessions.set("session_old", user("session_old"), { ttlMs: 0 });
    await sessions.set("session_new", user("session_new"), { ttlMs: 60_000 });

    await expect(sessions.get("session_old")).resolves.toBeNull();
    await expect(sessions.exists("session_old")).resolves.toBe(false);
    await expect(sessions.scanPrefix("session_")).resolves.toEqual([
      [normaliseKey("session_new"), user("session_new")],
    ]);

    const driver = memory();
    const namespace = normaliseNamespace("contract_ttl_direct");
    const collection = normaliseCollectionName("sessions");
    const key = normaliseKey("expired");
    await driver.set(namespace, collection, key, stored(user("expired"), 0), {});
    await expect(driver.get(namespace, collection, key, { allowExpired: true })).resolves.not.toBeNull();
  });

  it("matches error and capability contract", async () => {
    const driver = memory();
    expect(driver.capabilities()).toMatchObject({
      transactions: false,
      ttl: true,
      prefixScan: true,
      batchWrite: true,
      rawSql: false,
      documentQuery: false,
    });

    expect(() => normaliseKey("")).toThrow(YugenDbError);
    expect(() => normaliseNamespace("")).toThrow(YugenDbError);
    expect(() => normaliseCollectionName("")).toThrow(YugenDbError);

    try {
      normaliseKey("");
    } catch (error) {
      expect(isYugenDbError(error)).toBe(true);
      if (isYugenDbError(error)) {
        expect(error.code).toBe("INVALID_KEY");
      }
    }

    await expect(
      driver.delete(
        normaliseNamespace("contract_errors"),
        normaliseCollectionName("users"),
        normaliseKey("missing"),
        { mustExist: true },
      ),
    ).rejects.toMatchObject({ code: "NOT_FOUND" });
  });
});
