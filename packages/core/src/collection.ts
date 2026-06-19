import type { CollectionName, Key } from "./key";
import type { DeleteOptions, ReadOptions, ScanOptions, WriteOptions } from "./options";
import type { Store } from "./store";

/** Typed handle for values stored in one yugendb collection. */
export class Collection<T> {
  /** Collection name used by this handle. */
  readonly name: CollectionName;
  private readonly store: Store;

  constructor(store: Store, name: CollectionName) {
    this.store = store;
    this.name = name;
  }

  /** Reads a typed value from this collection. */
  get(key: string | Key, options: ReadOptions = {}): Promise<T | null> {
    return this.store.getFromCollection<T>(this.name, key, options);
  }

  /** Writes a typed value to this collection. */
  set(key: string | Key, value: T, options: WriteOptions = {}): Promise<void> {
    return this.store.setInCollection(this.name, key, value, options);
  }

  /** Deletes a value from this collection. */
  delete(key: string | Key, options: DeleteOptions = {}): Promise<boolean> {
    return this.store.deleteFromCollection(this.name, key, options);
  }

  /** Returns whether a non-expired value exists in this collection. */
  exists(key: string | Key): Promise<boolean> {
    return this.store.existsInCollection(this.name, key);
  }

  /** Scans typed values in this collection by key prefix. */
  scanPrefix(
    prefix: string,
    options: Omit<ScanOptions, "prefix"> = {},
  ): Promise<Array<[Key, T]>> {
    return this.store.scanPrefixInCollection<T>(this.name, prefix, options);
  }
}
