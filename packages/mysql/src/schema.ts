/** MySQL storage schema helpers for yugendb. */
export const storeTableName = "yugendb_store";
export const schemaVersion = 1;
export const migrationMetadataTableName = "yugendb_migrations";
export const valueColumnType = "LONGBLOB";

export const createStoreTableSql = `CREATE TABLE IF NOT EXISTS yugendb_store (
    namespace VARCHAR(512) NOT NULL,
    collection VARCHAR(512) NOT NULL,
    \`key\` TEXT NOT NULL,
    value LONGBLOB NOT NULL,
    codec TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT,
    PRIMARY KEY (namespace(191), collection(191), \`key\`(191))
);`;

export const createPrefixIndexSql =
  "CREATE INDEX idx_yugendb_store_prefix ON yugendb_store(namespace(191), collection(191), `key`(191));";
export const createExpiresAtIndexSql =
  "CREATE INDEX idx_yugendb_store_expires_at ON yugendb_store(expires_at(191));";

export const allSchemaStatements = [
  createStoreTableSql,
] as const;

/** Describes the MySQL storage schema used by the driver. */
export interface MysqlDriverStorageSchema {
  readonly version: number;
  readonly storeTableName: string;
  readonly migrationMetadataTableName: string;
  readonly valueColumnType: string;
}

/** Returns the current MySQL storage schema description. */
export function mysqlStorageSchema(): MysqlDriverStorageSchema {
  return { version: schemaVersion, storeTableName, migrationMetadataTableName, valueColumnType };
}
