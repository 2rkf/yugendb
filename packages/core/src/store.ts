import { createValueMetadata } from "./value";
import { Collection } from "./collection";
import { JsonCodec, type Codec } from "./codec";
import type { Capabilities } from "./capabilities";
import type { Driver } from "./driver";
import {
  DEFAULT_COLLECTION,
  DEFAULT_NAMESPACE,
  type CollectionName,
  type Key,
  type Namespace,
  normaliseCollectionName,
  normaliseKey,
  normaliseNamespace,
} from "./key";
import {
  type DeleteOptions,
  type ReadOptions,
  type ScanOptions,
  type WriteOptions,
  normaliseDeleteOptions,
  normaliseReadOptions,
  normaliseScanOptions,
  normaliseWriteOptions,
} from "./options";

/** Options used to create a typed yugendb store. */
export interface StoreOptions {
  /** Driver that provides the storage backend. */
  readonly driver: Driver;
  /** Namespace used by this store. */
  readonly namespace?: string | Namespace;
  /** Default collection used by direct store methods. */
  readonly collection?: string | CollectionName;
  /** Codec used to serialise and deserialise values. */
  readonly codec?: Codec;
}

/** User-facing typed store backed by a yugendb driver. */
export class Store {
  private readonly driver: Driver;
  private readonly codec: Codec;
  private readonly namespace: Namespace;
  private readonly defaultCollection: CollectionName;

  constructor(options: StoreOptions) {
    this.driver = options.driver;
    this.codec = options.codec ?? new JsonCodec();
    this.namespace = options.namespace !== undefined
      ? normaliseNamespace(options.namespace)
      : DEFAULT_NAMESPACE;
    this.defaultCollection = options.collection !== undefined
      ? normaliseCollectionName(options.collection)
      : DEFAULT_COLLECTION;
  }

  /** Opens any resources required by the store driver. */
  async initialise(): Promise<void> {
    await this.driver.initialise();
  }

  /** Closes resources owned by the store driver. */
  async finalise(): Promise<void> {
    await this.driver.finalise();
  }

  /** Returns the capabilities reported by the store driver. */
  capabilities(): Capabilities {
    return this.driver.capabilities();
  }

  /** Creates a typed handle for a named collection. */
  collection<T>(name: string | CollectionName): Collection<T> {
    return new Collection<T>(this, normaliseCollectionName(name));
  }

  /** Reads a typed value from the default collection. */
  async get<T>(key: string | Key, options: ReadOptions = {}): Promise<T | null> {
    return this.getFromCollection<T>(this.defaultCollection, key, options);
  }

  /** Writes a typed value to the default collection. */
  async set<T>(key: string | Key, value: T, options: WriteOptions = {}): Promise<void> {
    await this.setInCollection(this.defaultCollection, key, value, options);
  }

  /** Deletes a value from the default collection. */
  async delete(key: string | Key, options: DeleteOptions = {}): Promise<boolean> {
    return this.deleteFromCollection(this.defaultCollection, key, options);
  }

  /** Returns whether a non-expired value exists in the default collection. */
  async exists(key: string | Key): Promise<boolean> {
    return this.existsInCollection(this.defaultCollection, key);
  }

  /** Scans typed values in the default collection by key prefix. */
  async scanPrefix<T>(
    prefix: string,
    options: Omit<ScanOptions, "prefix"> = {},
  ): Promise<Array<[Key, T]>> {
    return this.scanPrefixInCollection<T>(this.defaultCollection, prefix, options);
  }

  /** Reads a typed value from a specific collection. */
  async getFromCollection<T>(
    collection: CollectionName,
    key: string | Key,
    options: ReadOptions = {},
  ): Promise<T | null> {
    const stored = await this.driver.get(
      this.namespace,
      collection,
      normaliseKey(key),
      normaliseReadOptions(options),
    );

    if (stored === null) {
      return null;
    }

    return this.codec.deserialise<T>(stored.bytes);
  }

  /** Writes a typed value to a specific collection. */
  async setInCollection<T>(
    collection: CollectionName,
    key: string | Key,
    value: T,
    options: WriteOptions = {},
  ): Promise<void> {
    const writeOptions = normaliseWriteOptions(options);
    const stored = {
      bytes: this.codec.serialise(value),
      metadata: createValueMetadata(this.codec.name, writeOptions.ttlMs),
    };

    await this.driver.set(
      this.namespace,
      collection,
      normaliseKey(key),
      stored,
      writeOptions,
    );
  }

  /** Deletes a value from a specific collection. */
  async deleteFromCollection(
    collection: CollectionName,
    key: string | Key,
    options: DeleteOptions = {},
  ): Promise<boolean> {
    return this.driver.delete(
      this.namespace,
      collection,
      normaliseKey(key),
      normaliseDeleteOptions(options),
    );
  }

  /** Returns whether a non-expired value exists in a specific collection. */
  async existsInCollection(collection: CollectionName, key: string | Key): Promise<boolean> {
    return this.driver.exists(this.namespace, collection, normaliseKey(key));
  }

  /** Scans typed values in a specific collection by key prefix. */
  async scanPrefixInCollection<T>(
    collection: CollectionName,
    prefix: string,
    options: Omit<ScanOptions, "prefix"> = {},
  ): Promise<Array<[Key, T]>> {
    const scanOptions = normaliseScanOptions({
      ...options,
      prefix: normaliseKey(prefix),
    });

    const rows = await this.driver.scanPrefix(this.namespace, collection, scanOptions);
    return rows.map(([key, stored]) => [key, this.codec.deserialise<T>(stored.bytes)]);
  }
}

/** Creates and initialises a typed yugendb store. */
export async function createStore(options: StoreOptions): Promise<Store> {
  const store = new Store(options);
  await store.initialise();
  return store;
}
