/** Options used when reading values. */
export interface ReadOptions {
  /** Return expired values instead of filtering them out. */
  readonly allowExpired?: boolean;
}

/** Options used when writing values. */
export interface WriteOptions {
  /** Time-to-live in milliseconds. */
  readonly ttlMs?: number;
  /** Replace an existing value when true. */
  readonly overwrite?: boolean;
}

/** Options used when deleting values. */
export interface DeleteOptions {
  /** Throw when the target value does not exist. */
  readonly mustExist?: boolean;
}

/** Options used when scanning by key prefix. */
export interface ScanOptions {
  /** Key prefix to scan. */
  readonly prefix?: string;
  /** Maximum number of results to return. */
  readonly limit?: number;
  /** Include expired values in scan results. */
  readonly includeExpired?: boolean;
}

/** Applies default read options. */
export function normaliseReadOptions(options: ReadOptions = {}): Required<ReadOptions> {
  return {
    allowExpired: options.allowExpired ?? false,
  };
}

/** Applies default write options. */
export function normaliseWriteOptions(options: WriteOptions = {}): WriteOptions {
  const normalised: WriteOptions = {
    overwrite: options.overwrite ?? true,
  };

  if (options.ttlMs !== undefined) {
    return { ...normalised, ttlMs: options.ttlMs };
  }

  return normalised;
}

/** Applies default delete options. */
export function normaliseDeleteOptions(options: DeleteOptions = {}): Required<DeleteOptions> {
  return {
    mustExist: options.mustExist ?? false,
  };
}

/** Applies default scan options. */
export function normaliseScanOptions(options: ScanOptions = {}): ScanOptions {
  const normalised: ScanOptions = {
    includeExpired: options.includeExpired ?? false,
  };

  return {
    ...normalised,
    ...(options.prefix !== undefined ? { prefix: options.prefix } : {}),
    ...(options.limit !== undefined ? { limit: options.limit } : {}),
  };
}
