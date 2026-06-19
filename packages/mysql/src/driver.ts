import { Buffer } from "node:buffer";
import { createPool, type Pool, type PoolConnection, type ResultSetHeader, type RowDataPacket } from "mysql2/promise";

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

import { allSchemaStatements, mysqlStorageSchema, type MysqlDriverStorageSchema } from "./schema.js";

/** Options used to connect the MySQL driver. */
export interface MysqlDriverOptions {
  readonly connectionString: string;
  readonly createSchemaOnInitialise?: boolean;
  readonly connectionLimit?: number;
}

/** MySQL driver for yugendb external-service storage. */
export class MysqlDriver implements Driver {
  readonly name = "mysql";
  readonly options: Required<MysqlDriverOptions>;
  private pool: Pool | null = null;

  constructor(options: MysqlDriverOptions | string) {
    this.options = typeof options === "string"
      ? { connectionString: options, createSchemaOnInitialise: true, connectionLimit: 5 }
      : {
          connectionString: options.connectionString,
          createSchemaOnInitialise: options.createSchemaOnInitialise ?? true,
          connectionLimit: options.connectionLimit ?? 5,
        };
  }

  capabilities(): Capabilities {
    return mysqlCapabilities();
  }

  schema(): MysqlDriverStorageSchema {
    return mysqlStorageSchema();
  }

  async initialise(): Promise<void> {
    if (this.pool !== null) {
      return;
    }

    const pool = createPool({
      uri: this.options.connectionString,
      connectionLimit: this.options.connectionLimit,
    });
    this.pool = pool;

    if (this.options.createSchemaOnInitialise) {
      for (const statement of allSchemaStatements) {
        await pool.execute(statement);
      }
    } else {
      await pool.execute("SELECT 1");
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
    const [rows] = await this.db().execute<RowDataPacket[]>(
      "SELECT value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` = ?",
      [namespace, collection, key],
    );
    if (rows.length === 0) {
      return null;
    }

    const value = rowToStoredValue(rows[0]!);
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
        message: "MySQL driver refused to overwrite an existing value.",
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
    const [result] = await this.db().execute<ResultSetHeader>(
      "DELETE FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` = ?",
      [namespace, collection, key],
    );
    const removed = result.affectedRows > 0;

    if (options.mustExist === true && !removed) {
      throw new YugenDbError({
        code: "NOT_FOUND",
        message: "MySQL driver could not delete a missing value.",
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
    const [rows] = await this.db().execute<RowDataPacket[]>(
      "SELECT `key`, value, codec, created_at, updated_at, expires_at FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` LIKE CONCAT(?, '%') ORDER BY `key` ASC",
      [namespace, collection, prefix],
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
    const connection = await this.db().getConnection();
    let appliedOperations = 0;
    let deletedKeys = 0;

    try {
      await connection.beginTransaction();
      for (const operation of batch.operations) {
        if (operation.kind === "set") {
          await insertValue(connection, operation.namespace, operation.collection, operation.key, operation.value);
          appliedOperations += 1;
        } else {
          const [result] = await connection.execute<ResultSetHeader>(
            "DELETE FROM yugendb_store WHERE namespace = ? AND collection = ? AND `key` = ?",
            [operation.namespace, operation.collection, operation.key],
          );
          if (result.affectedRows > 0) {
            deletedKeys += 1;
          }
          appliedOperations += 1;
        }
      }
      await connection.commit();
    } catch (error) {
      await connection.rollback();
      throw error;
    } finally {
      connection.release();
    }

    return { appliedOperations, deletedKeys };
  }

  private db(): Pool {
    if (this.pool === null) {
      throw new YugenDbError({ code: "CONNECTION_ERROR", message: "MySQL driver has not been initialised." });
    }
    return this.pool;
  }
}

/** Returns the capabilities reported by the MySQL driver. */
export function mysqlCapabilities(): Capabilities {
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

/** Creates a MySQL yugendb driver. */
export function mysql(
  connectionString: string,
  options: Omit<MysqlDriverOptions, "connectionString"> = {},
): MysqlDriver {
  return new MysqlDriver({ connectionString, ...options });
}

async function insertValue(
  client: Pool | PoolConnection,
  namespace: Namespace,
  collection: CollectionName,
  key: Key,
  value: StoredValue,
): Promise<void> {
  await client.execute(
    `INSERT INTO yugendb_store
(namespace, collection, \`key\`, value, codec, created_at, updated_at, expires_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON DUPLICATE KEY UPDATE
  value = VALUES(value),
  codec = VALUES(codec),
  updated_at = VALUES(updated_at),
  expires_at = VALUES(expires_at)`,
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

function rowToStoredValue(row: RowDataPacket): StoredValue {
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
