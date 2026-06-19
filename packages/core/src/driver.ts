import type { Batch, BatchResult } from "./batch";
import type { Capabilities } from "./capabilities";
import type { CollectionName, Key, Namespace } from "./key";
import type { DeleteOptions, ReadOptions, ScanOptions, WriteOptions } from "./options";
import type { StoredValue } from "./value";

/** Low-level storage contract implemented by yugendb drivers. */
export interface Driver {
  /** Stable driver name. */
  readonly name: string;
  /** Returns the features supported by this driver. */
  capabilities(): Capabilities;

  /** Opens any resources required by the driver. */
  initialise(): Promise<void>;
  /** Closes resources owned by the driver. */
  finalise(): Promise<void>;

  /** Reads a stored value by namespace, collection, and key. */
  get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null>;

  /** Writes a stored value by namespace, collection, and key. */
  set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): Promise<void>;

  /** Deletes a stored value by namespace, collection, and key. */
  delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean>;

  /** Returns whether a non-expired value exists. */
  exists(namespace: Namespace, collection: CollectionName, key: Key): Promise<boolean>;

  /** Scans stored values whose keys start with a prefix. */
  scanPrefix(
    namespace: Namespace,
    collection: CollectionName,
    options: ScanOptions,
  ): Promise<Array<[Key, StoredValue]>>;

  /** Applies a batch of write operations. */
  batch(batch: Batch): Promise<BatchResult>;
}
