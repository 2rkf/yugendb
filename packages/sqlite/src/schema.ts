/** SQLite schema helpers for the yugendb storage table. */
export const SQLITE_SCHEMA_VERSION = 1;
export const SQLITE_STORE_TABLE_NAME = "yugendb_store";
export const SQLITE_MIGRATION_METADATA_TABLE_NAME = "yugendb_schema_migrations";

export const CREATE_STORE_TABLE_SQL = `CREATE TABLE IF NOT EXISTS yugendb_store (
    namespace TEXT NOT NULL,
    collection TEXT NOT NULL,
    key TEXT NOT NULL,
    value BLOB NOT NULL,
    codec TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT,
    PRIMARY KEY (namespace, collection, key)
);`;

export const CREATE_PREFIX_INDEX_SQL = `CREATE INDEX IF NOT EXISTS idx_yugendb_store_prefix
ON yugendb_store(namespace, collection, key);`;

export const CREATE_EXPIRES_AT_INDEX_SQL = `CREATE INDEX IF NOT EXISTS idx_yugendb_store_expires_at
ON yugendb_store(expires_at);`;

export const CREATE_MIGRATION_METADATA_TABLE_SQL = `CREATE TABLE IF NOT EXISTS yugendb_schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);`;

export const SQLITE_SCHEMA_STATEMENTS = [
  CREATE_STORE_TABLE_SQL,
  CREATE_PREFIX_INDEX_SQL,
  CREATE_EXPIRES_AT_INDEX_SQL,
  CREATE_MIGRATION_METADATA_TABLE_SQL,
] as const;

/** Describes the SQLite storage schema used by the driver. */
export interface SqliteStorageSchema {
  readonly version: number;
  readonly storeTableName: string;
  readonly migrationMetadataTableName: string;
  readonly statements: readonly string[];
}

/** Returns the current SQLite storage schema description. */
export function sqliteStorageSchema(): SqliteStorageSchema {
  return {
    version: SQLITE_SCHEMA_VERSION,
    storeTableName: SQLITE_STORE_TABLE_NAME,
    migrationMetadataTableName: SQLITE_MIGRATION_METADATA_TABLE_NAME,
    statements: SQLITE_SCHEMA_STATEMENTS,
  };
}
