/** In-memory TypeScript driver for yugendb. */
import {
  type Batch,
  type BatchOperation,
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
  type ValueMetadata,
  type WriteOptions,
  YugenDbError,
  isExpired,
  memoryCapabilities,
} from "@yugendb/core";

/** Options for the in-memory TypeScript driver. */
export interface MemoryDriverOptions {
  /**
   * Clone stored values on write and read to reduce accidental mutation.
   * This is enabled by default to keep in-memory reads and writes isolated.
   */
  readonly cloneValues?: boolean;
}

const STORAGE_SEPARATOR = "\u001f";

/** In-process yugendb driver for temporary local storage. */
export class MemoryDriver implements Driver {
  readonly name = "memory";

  private values = new Map<string, StoredValue>();
  private readonly cloneValues: boolean;

  constructor(options: MemoryDriverOptions = {}) {
    this.cloneValues = options.cloneValues ?? true;
  }

  capabilities(): Capabilities {
    return memoryCapabilities();
  }

  async initialise(): Promise<void> {
    // The memory driver owns no external resources.
  }

  async finalise(): Promise<void> {
    // Keep stored values available until clear() or process exit.
  }

  async get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null> {
    const stored = this.values.get(this.storageKey(namespace, collection, key));

    if (stored === undefined) {
      return null;
    }

    if (!options.allowExpired && isExpired(stored)) {
      return null;
    }

    return this.cloneStoredValue(stored);
  }

  async set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): Promise<void> {
    this.setOn(this.values, namespace, collection, key, value, options);
  }

  async delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean> {
    return this.deleteOn(this.values, namespace, collection, key, options);
  }

  async exists(namespace: Namespace, collection: CollectionName, key: Key): Promise<boolean> {
    const stored = this.values.get(this.storageKey(namespace, collection, key));
    return stored !== undefined && !isExpired(stored);
  }

  async scanPrefix(
    namespace: Namespace,
    collection: CollectionName,
    options: ScanOptions,
  ): Promise<Array<[Key, StoredValue]>> {
    if (options.limit !== undefined && (!Number.isInteger(options.limit) || options.limit < 0)) {
      throw new YugenDbError({
        code: "INVALID_VALUE",
        message: "Invalid scan option: limit must be a non-negative integer.",
      });
    }

    const prefix = options.prefix ?? "";
    const rows: Array<[Key, StoredValue]> = [];

    for (const [storageKey, stored] of this.values.entries()) {
      const parsed = this.parseStorageKey(storageKey);

      if (parsed === null) {
        continue;
      }

      if (parsed.namespace !== namespace || parsed.collection !== collection) {
        continue;
      }

      if (!parsed.key.startsWith(prefix)) {
        continue;
      }

      if (!options.includeExpired && isExpired(stored)) {
        continue;
      }

      rows.push([parsed.key as Key, this.cloneStoredValue(stored)]);
    }

    rows.sort(([left], [right]) => left.localeCompare(right));
    return options.limit === undefined ? rows : rows.slice(0, options.limit);
  }

  async batch(batch: Batch): Promise<BatchResult> {
    const nextValues = new Map(this.values);
    let deletedKeys = 0;

    for (const operation of batch.operations) {
      if (operation.kind === "set") {
        this.setOn(
          nextValues,
          operation.namespace,
          operation.collection,
          operation.key,
          operation.value,
          operation.options ?? {},
        );
        continue;
      }

      if (operation.kind === "delete") {
        const deleted = this.deleteOn(
          nextValues,
          operation.namespace,
          operation.collection,
          operation.key,
          operation.options ?? {},
        );

        if (deleted) {
          deletedKeys += 1;
        }
      }
    }

    this.values = nextValues;

    return {
      appliedOperations: batch.operations.length,
      deletedKeys,
    };
  }

  clear(): void {
    this.values.clear();
  }

  private setOn(
    values: Map<string, StoredValue>,
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): void {
    const id = this.storageKey(namespace, collection, key);
    const existing = values.get(id);

    if (options.overwrite === false && existing !== undefined && !isExpired(existing)) {
      throw new YugenDbError({
        code: "CONFLICT",
        message: "A value already exists for this namespace, collection, and key.",
      });
    }

    values.set(id, this.cloneStoredValue(this.applyTtlIfMissing(value, options)));
  }

  private deleteOn(
    values: Map<string, StoredValue>,
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): boolean {
    const deleted = values.delete(this.storageKey(namespace, collection, key));

    if (!deleted && options.mustExist === true) {
      throw new YugenDbError({
        code: "NOT_FOUND",
        message: "No value exists for the requested namespace, collection, and key.",
      });
    }

    return deleted;
  }

  private applyTtlIfMissing(value: StoredValue, options: WriteOptions): StoredValue {
    if (options.ttlMs === undefined || value.metadata.expiresAt instanceof Date) {
      return value;
    }

    if (!Number.isFinite(options.ttlMs) || options.ttlMs < 0) {
      throw new YugenDbError({
        code: "INVALID_VALUE",
        message: "Invalid write option: ttlMs must be a non-negative finite number.",
      });
    }

    const now = new Date();
    return {
      bytes: value.bytes,
      metadata: {
        ...value.metadata,
        updatedAt: now,
        expiresAt: new Date(now.getTime() + options.ttlMs),
      },
    };
  }

  private cloneStoredValue(value: StoredValue): StoredValue {
    if (!this.cloneValues) {
      return value;
    }

    const metadata = this.cloneMetadata(value.metadata);
    return {
      bytes: new Uint8Array(value.bytes),
      metadata,
    };
  }

  private cloneMetadata(metadata: ValueMetadata): ValueMetadata {
    const cloned: ValueMetadata = {
      codec: metadata.codec,
      createdAt: new Date(metadata.createdAt.getTime()),
      updatedAt: new Date(metadata.updatedAt.getTime()),
    };

    if (metadata.expiresAt !== undefined) {
      return {
        ...cloned,
        expiresAt: metadata.expiresAt === null
          ? null
          : new Date(metadata.expiresAt.getTime()),
      };
    }

    return cloned;
  }

  private storageKey(namespace: Namespace, collection: CollectionName, key: Key): string {
    return `${namespace}${STORAGE_SEPARATOR}${collection}${STORAGE_SEPARATOR}${key}`;
  }

  private parseStorageKey(storageKey: string): {
    readonly namespace: string;
    readonly collection: string;
    readonly key: string;
  } | null {
    const parts = storageKey.split(STORAGE_SEPARATOR);

    if (parts.length !== 3) {
      return null;
    }

    const [namespace, collection, key] = parts;

    if (namespace === undefined || collection === undefined || key === undefined) {
      return null;
    }

    return { namespace, collection, key };
  }
}

/** Creates a new in-memory yugendb driver. */
export function memory(options: MemoryDriverOptions = {}): MemoryDriver {
  return new MemoryDriver(options);
}

/** Batch operation type re-exported for driver users. */
export type { BatchOperation };
