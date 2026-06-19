/** PostgreSQL storage schema helpers for yugendb. */
export const storeTableName = "yugendb_store";
export const schemaVersion = 1;
export const migrationMetadataTableName = "yugendb_migrations";
export const valueColumnType = "BYTEA";

export const createStoreTableSql = `CREATE TABLE IF NOT EXISTS yugendb_store (
    namespace TEXT NOT NULL,
    collection TEXT NOT NULL,
    key TEXT NOT NULL,
    value BYTEA NOT NULL,
    codec TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT,
    PRIMARY KEY (namespace, collection, key)
);`;

export const createPrefixIndexSql =
  "CREATE INDEX IF NOT EXISTS idx_yugendb_store_prefix ON yugendb_store(namespace, collection, key);";
export const createExpiresAtIndexSql =
  "CREATE INDEX IF NOT EXISTS idx_yugendb_store_expires_at ON yugendb_store(expires_at);";

export const allSchemaStatements = [
  createStoreTableSql,
  createPrefixIndexSql,
  createExpiresAtIndexSql,
] as const;

/** Describes the PostgreSQL storage schema used by the driver. */
export interface PostgresDriverStorageSchema {
  readonly version: number;
  readonly storeTableName: string;
  readonly migrationMetadataTableName: string;
  readonly valueColumnType: string;
}

/** Returns the current PostgreSQL storage schema description. */
export function postgresStorageSchema(): PostgresDriverStorageSchema {
  return {
    version: schemaVersion,
    storeTableName,
    migrationMetadataTableName,
    valueColumnType,
  };
}
