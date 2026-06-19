import { existsSync } from "node:fs";
import { createRequire } from "node:module";

import {
  type Batch,
  type BatchResult,
  type Capabilities,
  type CollectionName,
  type DeleteOptions,
  type Driver,
  type Key,
  type Namespace,
  type ReadOptions,
  type ScanOptions,
  type StoredValue,
  type WriteOptions,
  YugenDbError,
  isExpired,
  normaliseKey,
  sqliteCapabilities as coreSqliteCapabilities,
} from "@yugendb/core";

import { type SqliteStorageSchema, sqliteStorageSchema } from "./schema.js";

type DatabaseSync = {
  exec(sql: string): void;
  close(): void;
  prepare(sql: string): {
    get(...values: unknown[]): unknown;
    all(...values: unknown[]): unknown[];
    run(...values: unknown[]): { changes?: number } | void;
  };
};

type DatabaseSyncConstructor = new (path: string) => DatabaseSync;

const require = createRequire(import.meta.url);

/** Options used to open the SQLite driver. */
export interface SqliteDriverOptions {
  readonly path: string;
  readonly createIfMissing?: boolean;
}

interface StoredValueRow {
  readonly key?: string;
  readonly value: Uint8Array | Buffer;
  readonly codec: string;
  readonly created_at: string;
  readonly updated_at: string;
  readonly expires_at: string | null;
}

/** SQLite driver for local durable yugendb storage. */
export class SqliteDriver implements Driver {
  readonly name = "sqlite";
  readonly options: Required<SqliteDriverOptions>;
  private database: DatabaseSync | null = null;

  constructor(options: SqliteDriverOptions | string) {
    this.options = typeof options === "string"
      ? { path: options, createIfMissing: true }
      : { path: options.path, createIfMissing: options.createIfMissing ?? true };
  }

  capabilities(): Capabilities {
    return sqliteCapabilities();
  }


  schema(): SqliteStorageSchema {
    return sqliteStorageSchema();
  }

  async initialise(): Promise<void> {
    if (this.database !== null) {
      return;
    }

    let DatabaseSync: DatabaseSyncConstructor;
    try {
      ({ DatabaseSync } = require("node:sqlite") as { DatabaseSync: DatabaseSyncConstructor });
    } catch (error) {
      throw new YugenDbError({
        code: "UNSUPPORTED_FEATURE",
        message: "SQLite storage requires a Node.js runtime with node:sqlite support.",
        cause: error,
      });
    }

    if (!this.options.createIfMissing && this.options.path !== ":memory:" && !existsSync(this.options.path)) {
      throw new YugenDbError({
        code: "CONNECTION_ERROR",
        message: `SQLite database does not exist: ${this.options.path}`,
      });
    }

    const database = new DatabaseSync(this.options.path);
    for (const statement of sqliteStorageSchema().statements) {
      database.exec(statement);
    }
    this.database = database;
  }

  async finalise(): Promise<void> {
    this.database?.close();
    this.database = null;
  }

  async get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null> {
    const row = this.db()
      .prepare("SELECT value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ? AND collection = ? AND key = ?")
      .get(namespace, collection, key) as StoredValueRow | undefined;

    if (row === undefined) {
      return null;
    }

    const value = rowToStoredValue(row);
    if (isExpired(value) && options.allowExpired !== true) {
      return null;
    }
    return value;
  }

  async set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): Promise<void> {
    if (options.overwrite === false) {
      const existing = await this.get(namespace, collection, key, { allowExpired: false });
      if (existing !== null) {
        throw new YugenDbError({
          code: "CONFLICT",
          message: "SQLite driver refused to overwrite an existing value.",
        });
      }
    }

    this.db()
      .prepare(`INSERT INTO yugendb_store
(namespace, collection, key, value, codec, created_at, updated_at, expires_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(namespace, collection, key) DO UPDATE SET
  value = excluded.value,
  codec = excluded.codec,
  updated_at = excluded.updated_at,
  expires_at = excluded.expires_at`)
      .run(
        namespace,
        collection,
        key,
        Buffer.from(value.bytes),
        value.metadata.codec,
        value.metadata.createdAt.toISOString(),
        value.metadata.updatedAt.toISOString(),
        value.metadata.expiresAt instanceof Date ? value.metadata.expiresAt.toISOString() : null,
      );
  }

  async delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean> {
    const result = this.db()
      .prepare("DELETE FROM yugendb_store WHERE namespace = ? AND collection = ? AND key = ?")
      .run(namespace, collection, key) as { changes?: number } | undefined;
    const removed = (result?.changes ?? 0) > 0;

    if (options.mustExist === true && !removed) {
      throw new YugenDbError({
        code: "NOT_FOUND",
        message: "SQLite driver could not delete a missing value.",
      });
    }

    return removed;
  }

  async exists(namespace: Namespace, collection: CollectionName, key: Key): Promise<boolean> {
    return (await this.get(namespace, collection, key, { allowExpired: false })) !== null;
  }

  async scanPrefix(
    namespace: Namespace,
    collection: CollectionName,
    options: ScanOptions,
  ): Promise<Array<[Key, StoredValue]>> {
    const prefix = options.prefix ?? "";
    const rows = this.db()
      .prepare("SELECT key, value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ? AND collection = ? AND key LIKE ? ORDER BY key ASC")
      .all(namespace, collection, `${prefix}%`) as StoredValueRow[];

    const values: Array<[Key, StoredValue]> = [];
    for (const row of rows) {
      const value = rowToStoredValue(row);
      if (isExpired(value) && options.includeExpired !== true) {
        continue;
      }
      values.push([normaliseKey(row.key ?? ""), value]);
      if (options.limit !== undefined && values.length >= options.limit) {
        break;
      }
    }
    return values;
  }

  async batch(batch: Batch): Promise<BatchResult> {
    this.db().exec("BEGIN IMMEDIATE");
    let appliedOperations = 0;
    let deletedKeys = 0;

    try {
      for (const operation of batch.operations) {
        if (operation.kind === "set") {
          await this.set(
            operation.namespace,
            operation.collection,
            operation.key,
            operation.value,
            operation.options ?? {},
          );
          appliedOperations += 1;
        } else {
          const deleted = await this.delete(
            operation.namespace,
            operation.collection,
            operation.key,
            operation.options ?? {},
          );
          if (deleted) {
            deletedKeys += 1;
          }
          appliedOperations += 1;
        }
      }
      this.db().exec("COMMIT");
    } catch (error) {
      this.db().exec("ROLLBACK");
      throw error;
    }

    return { appliedOperations, deletedKeys };
  }

  private db(): DatabaseSync {
    if (this.database === null) {
      throw new YugenDbError({
        code: "CONNECTION_ERROR",
        message: "SQLite driver has not been initialised.",
      });
    }
    return this.database;
  }
}

/** Capabilities for the implemented SQLite driver. */
export function sqliteCapabilities(): Capabilities {
  return {
    ...coreSqliteCapabilities(),
    rawSql: false,
    jsonQuery: false,
    backup: false,
  };
}

/** Creates a SQLite yugendb driver. */
export function sqlite(
  path: string,
  options: Omit<SqliteDriverOptions, "path"> = {},
): SqliteDriver {
  return new SqliteDriver({ path, ...options });
}

function rowToStoredValue(row: StoredValueRow): StoredValue {
  return {
    bytes: new Uint8Array(row.value),
    metadata: {
      codec: row.codec,
      createdAt: new Date(row.created_at),
      updatedAt: new Date(row.updated_at),
      expiresAt: row.expires_at === null ? null : new Date(row.expires_at),
    },
  };
}
