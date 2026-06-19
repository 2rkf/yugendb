import type { CollectionName, Key, Namespace } from "./key";
import type { DeleteOptions, WriteOptions } from "./options";
import type { StoredValue } from "./value";

/** A write operation that can be applied as part of a batch. */
export type BatchOperation =
  | {
      /** Store or replace a value. */
      readonly kind: "set";
      /** Target namespace. */
      readonly namespace: Namespace;
      /** Target collection. */
      readonly collection: CollectionName;
      /** Target key. */
      readonly key: Key;
      /** Value to store. */
      readonly value: StoredValue;
      /** Optional write behaviour. */
      readonly options?: WriteOptions;
    }
  | {
      /** Delete a value. */
      readonly kind: "delete";
      /** Target namespace. */
      readonly namespace: Namespace;
      /** Target collection. */
      readonly collection: CollectionName;
      /** Target key. */
      readonly key: Key;
      /** Optional delete behaviour. */
      readonly options?: DeleteOptions;
    };

/** Ordered set of write operations. */
export interface Batch {
  /** Operations applied in order. */
  readonly operations: readonly BatchOperation[];
}

/** Result returned after applying a batch. */
export interface BatchResult {
  /** Number of operations applied. */
  readonly appliedOperations: number;
  /** Number of keys deleted, when reported by the driver. */
  readonly deletedKeys?: number;
}

/** Creates a batch from the provided operations. */
export function createBatch(operations: readonly BatchOperation[] = []): Batch {
  return { operations };
}
