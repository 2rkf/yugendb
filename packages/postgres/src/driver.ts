import { Buffer } from "node:buffer";
import { Pool, type PoolClient } from "pg";

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
  minimalCapabilities,
  normaliseKey,
} from "@yugendb/core";

import { allSchemaStatements, postgresStorageSchema, type PostgresDriverStorageSchema } from "./schema.js";

/** Options used to connect the PostgreSQL driver. */
export interface PostgresDriverOptions {
  readonly connectionString: string;
  readonly createSchemaOnInitialise?: boolean;
  readonly maxConnections?: number;
}

/** PostgreSQL driver for yugendb external-service storage. */
export class PostgresDriver implements Driver {
  readonly name = "postgres";
  readonly options: Required<PostgresDriverOptions>;
  private pool: Pool | null = null;

  constructor(options: PostgresDriverOptions | string) {
    this.options = typeof options === "string"
      ? { connectionString: options, createSchemaOnInitialise: true, maxConnections: 5 }
      : {
          connectionString: options.connectionString,
          createSchemaOnInitialise: options.createSchemaOnInitialise ?? true,
          maxConnections: options.maxConnections ?? 5,
        };
  }

  capabilities(): Capabilities {
    return postgresCapabilities();
  }

  schema(): PostgresDriverStorageSchema {
    return postgresStorageSchema();
  }

  async initialise(): Promise<void> {
    if (this.pool !== null) {
      return;
    }

    this.pool = new Pool({ connectionString: this.options.connectionString, max: this.options.maxConnections });
    if (this.options.createSchemaOnInitialise) {
      for (const statement of allSchemaStatements) {
        await this.pool.query(statement);
      }
    } else {
      await this.pool.query("SELECT 1");
    }
  }

  async finalise(): Promise<void> {
    await this.pool?.end();
    this.pool = null;
  }

  async get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null> {
    const { rows } = await this.db().query(
      "SELECT value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = $1 AND collection = $2 AND key = $3",
      [namespace, collection, key],
    );
    if (rows.length === 0) {
      return null;
    }

    const value = rowToStoredValue(rows[0]);
    return isExpired(value) && options.allowExpired !== true ? null : value;
  }

  async set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): Promise<void> {
    if (options.overwrite === false && await this.exists(namespace, collection, key)) {
      throw new YugenDbError({
        code: "CONFLICT",
        message: "PostgreSQL driver refused to overwrite an existing value.",
      });
    }
    await insertValue(this.db(), namespace, collection, key, value);
  }

  async delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean> {
    const result = await this.db().query(
      "DELETE FROM yugendb_store WHERE namespace = $1 AND collection = $2 AND key = $3",
      [namespace, collection, key],
    );
    const removed = (result.rowCount ?? 0) > 0;

    if (options.mustExist === true && !removed) {
      throw new YugenDbError({
        code: "NOT_FOUND",
        message: "PostgreSQL driver could not delete a missing value.",
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
    const { rows } = await this.db().query(
      "SELECT key, value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = $1 AND collection = $2 AND key LIKE $3 ORDER BY key ASC",
      [namespace, collection, `${prefix}%`],
    );
    const values: Array<[Key, StoredValue]> = [];

    for (const row of rows) {
      const value = rowToStoredValue(row);
      if (isExpired(value) && options.includeExpired !== true) {
        continue;
      }
      values.push([normaliseKey(row.key), value]);
      if (options.limit !== undefined && values.length >= options.limit) {
        break;
      }
    }

    return values;
  }

  async batch(batch: Batch): Promise<BatchResult> {
    const client = await this.db().connect();
    let appliedOperations = 0;
    let deletedKeys = 0;

    try {
      await client.query("BEGIN");
      for (const operation of batch.operations) {
        if (operation.kind === "set") {
          await insertValue(client, operation.namespace, operation.collection, operation.key, operation.value);
          appliedOperations += 1;
        } else {
          const result = await client.query(
            "DELETE FROM yugendb_store WHERE namespace = $1 AND collection = $2 AND key = $3",
            [operation.namespace, operation.collection, operation.key],
          );
          if ((result.rowCount ?? 0) > 0) {
            deletedKeys += 1;
          }
          appliedOperations += 1;
        }
      }
      await client.query("COMMIT");
    } catch (error) {
      await client.query("ROLLBACK");
      throw error;
    } finally {
      client.release();
    }

    return { appliedOperations, deletedKeys };
  }

  private db(): Pool {
    if (this.pool === null) {
      throw new YugenDbError({ code: "CONNECTION_ERROR", message: "PostgreSQL driver has not been initialised." });
    }
    return this.pool;
  }
}

/** Returns the capabilities reported by the PostgreSQL driver. */
export function postgresCapabilities(): Capabilities {
  return {
    ...minimalCapabilities(),
    transactions: true,
    ttl: true,
    prefixScan: true,
    batchWrite: true,
    migrations: true,
    connectionPooling: true,
  };
}

/** Creates a PostgreSQL yugendb driver. */
export function postgres(
  connectionString: string,
  options: Omit<PostgresDriverOptions, "connectionString"> = {},
): PostgresDriver {
  return new PostgresDriver({ connectionString, ...options });
}

async function insertValue(
  client: Pool | PoolClient,
  namespace: Namespace,
  collection: CollectionName,
  key: Key,
  value: StoredValue,
): Promise<void> {
  await client.query(
    `INSERT INTO yugendb_store
(namespace, collection, key, value, codec, created_at, updated_at, expires_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT(namespace, collection, key) DO UPDATE SET
  value = excluded.value,
  codec = excluded.codec,
  updated_at = excluded.updated_at,
  expires_at = excluded.expires_at`,
    [
      namespace,
      collection,
      key,
      Buffer.from(value.bytes),
      value.metadata.codec,
      value.metadata.createdAt.toISOString(),
      value.metadata.updatedAt.toISOString(),
      value.metadata.expiresAt instanceof Date ? value.metadata.expiresAt.toISOString() : null,
    ],
  );
}

function rowToStoredValue(row: { value: Uint8Array | Buffer; codec: string; created_at: string; updated_at: string; expires_at: string | null }): StoredValue {
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
