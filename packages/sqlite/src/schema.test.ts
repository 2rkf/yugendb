import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { describe, expect, it } from "vitest";
import {
  createBatch,
  createValueMetadata,
  normaliseCollectionName,
  normaliseKey,
  normaliseNamespace,
} from "@yugendb/core";
import {
  CREATE_EXPIRES_AT_INDEX_SQL,
  CREATE_PREFIX_INDEX_SQL,
  CREATE_STORE_TABLE_SQL,
  SQLITE_SCHEMA_STATEMENTS,
  SQLITE_STORE_TABLE_NAME,
  SqliteDriver,
  sqlite,
  sqliteCapabilities,
} from "./index";

function tempDatabasePath(): { path: string; cleanup: () => void } {
  const directory = mkdtempSync(join(tmpdir(), "yugendb-sqlite-"));
  return {
    path: join(directory, "store.sqlite"),
    cleanup: () => rmSync(directory, { recursive: true, force: true }),
  };
}

function stored(text: string) {
  return {
    bytes: new TextEncoder().encode(text),
    metadata: createValueMetadata("json"),
  };
}

describe("SQLite schema", () => {
  it("exposes the storage table SQL", () => {
    expect(CREATE_STORE_TABLE_SQL).toContain("CREATE TABLE IF NOT EXISTS yugendb_store");
    expect(CREATE_STORE_TABLE_SQL).toContain("value BLOB NOT NULL");
    expect(CREATE_STORE_TABLE_SQL).toContain("PRIMARY KEY (namespace, collection, key)");
  });

  it("exposes prefix and expiry index SQL", () => {
    expect(CREATE_PREFIX_INDEX_SQL).toContain("idx_yugendb_store_prefix");
    expect(CREATE_EXPIRES_AT_INDEX_SQL).toContain("idx_yugendb_store_expires_at");
    expect(SQLITE_SCHEMA_STATEMENTS.some((statement) => statement.includes(SQLITE_STORE_TABLE_NAME))).toBe(true);
  });
});

describe("SqliteDriver", () => {
  it("reports its name, options, and implemented capabilities", () => {
    const driver = new SqliteDriver("app.db");
    expect(driver.name).toBe("sqlite");
    expect(driver.options).toEqual({ path: "app.db", createIfMissing: true });
    expect(sqliteCapabilities().prefixScan).toBe(true);
    expect(sqliteCapabilities().batchWrite).toBe(true);
  });

  it("sets, gets, scans, batches, and deletes values", async () => {
    const database = tempDatabasePath();
    const driver = sqlite(database.path);
    const namespace = normaliseNamespace("tests");
    const collection = normaliseCollectionName("users");
    const userA = normaliseKey("user_a");
    const userB = normaliseKey("user_b");

    try {
      await driver.initialise();
      expect(await driver.get(namespace, collection, userA, {})).toBeNull();

      await driver.set(namespace, collection, userB, stored("b"), {});
      await driver.set(namespace, collection, userA, stored("a"), {});

      const loaded = await driver.get(namespace, collection, userA, {});
      expect(new TextDecoder().decode(loaded?.bytes)).toBe("a");
      expect(await driver.exists(namespace, collection, userA)).toBe(true);

      const matches = await driver.scanPrefix(namespace, collection, { prefix: "user_" });
      expect(matches.map(([key]) => key)).toEqual(["user_a", "user_b"]);

      const batch = createBatch([
        {
          kind: "set",
          namespace,
          collection,
          key: normaliseKey("user_c"),
          value: stored("c"),
        },
        {
          kind: "delete",
          namespace,
          collection,
          key: userB,
        },
      ]);
      const result = await driver.batch(batch);
      expect(result.appliedOperations).toBe(2);
      expect(result.deletedKeys).toBe(1);
      expect(await driver.get(namespace, collection, userB, {})).toBeNull();

      expect(await driver.delete(namespace, collection, userA, {})).toBe(true);
    } finally {
      await driver.finalise();
      database.cleanup();
    }
  });

  it("hides expired values by default", async () => {
    const database = tempDatabasePath();
    const driver = sqlite(database.path);
    const namespace = normaliseNamespace("tests");
    const collection = normaliseCollectionName("sessions");
    const key = normaliseKey("session_001");

    try {
      await driver.initialise();
      await driver.set(
        namespace,
        collection,
        key,
        {
          bytes: new TextEncoder().encode("expired"),
          metadata: {
            codec: "json",
            createdAt: new Date(),
            updatedAt: new Date(),
            expiresAt: new Date(Date.now() - 1_000),
          },
        },
        {},
      );

      expect(await driver.get(namespace, collection, key, {})).toBeNull();
      expect(await driver.exists(namespace, collection, key)).toBe(false);
      expect(await driver.get(namespace, collection, key, { allowExpired: true })).not.toBeNull();
    } finally {
      await driver.finalise();
      database.cleanup();
    }
  });
});
