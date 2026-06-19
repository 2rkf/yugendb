/** Feature flags reported by a yugendb driver. */
export interface Capabilities {
  /** Driver supports transactional operation groups. */
  readonly transactions: boolean;
  /** Driver supports time-to-live expiry. */
  readonly ttl: boolean;
  /** Driver supports prefix scans. */
  readonly prefixScan: boolean;
  /** Driver supports atomic increment operations. */
  readonly atomicIncrement: boolean;
  /** Driver supports batch writes. */
  readonly batchWrite: boolean;
  /** Driver exposes a raw SQL escape hatch. */
  readonly rawSql: boolean;
  /** Driver exposes a document query escape hatch. */
  readonly documentQuery: boolean;
  /** Driver exposes JSON query behaviour. */
  readonly jsonQuery: boolean;
  /** Driver supports migration helpers. */
  readonly migrations: boolean;
  /** Driver manages connection pooling. */
  readonly connectionPooling: boolean;
  /** Driver supports watch or subscription behaviour. */
  readonly watch: boolean;
  /** Driver supports backup helpers. */
  readonly backup: boolean;
}

/** Returns the minimal capability set required of every driver. */
export function minimalCapabilities(): Capabilities {
  return {
    transactions: false,
    ttl: false,
    prefixScan: false,
    atomicIncrement: false,
    batchWrite: false,
    rawSql: false,
    documentQuery: false,
    jsonQuery: false,
    migrations: false,
    connectionPooling: false,
    watch: false,
    backup: false,
  };
}

/** Returns the capability set for the in-memory driver. */
export function memoryCapabilities(): Capabilities {
  return {
    ...minimalCapabilities(),
    ttl: true,
    prefixScan: true,
    batchWrite: true,
  };
}

/** Returns the capability set for the SQLite driver. */
export function sqliteCapabilities(): Capabilities {
  return {
    ...minimalCapabilities(),
    transactions: true,
    ttl: true,
    prefixScan: true,
    batchWrite: true,
    migrations: true,
  };
}
