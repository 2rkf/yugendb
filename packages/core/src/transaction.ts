import type { CollectionName, Key, Namespace } from "./key";
import type { DeleteOptions, ReadOptions, WriteOptions } from "./options";
import type { StoredValue } from "./value";

/** Transaction-like interface for drivers that can group operations. */
export interface Transaction {
  /** Reads a stored value within the transaction. */
  get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null>;

  /** Writes a stored value within the transaction. */
  set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): Promise<void>;

  /** Deletes a stored value within the transaction. */
  delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean>;

  /** Commits all pending transaction operations. */
  commit(): Promise<void>;
  /** Rolls back all pending transaction operations. */
  rollback(): Promise<void>;
}
