import { Buffer } from "node:buffer";
import { Binary, MongoClient, type Collection as MongoCollection } from "mongodb";

import {
  type Batch,
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
  type WriteOptions,
  YugenDbError,
  isExpired,
  minimalCapabilities,
  normaliseKey,
} from "@yugendb/core";

import { mongodbDocumentSchema, storageCollectionName, type MongoDbDocumentSchema } from "./schema.js";

/** Options used to connect the MongoDB driver. */
export interface MongoDbDriverOptions {
  readonly connectionString: string;
  readonly database?: string;
  readonly storageCollection?: string;
  readonly createIndexesOnInitialise?: boolean;
}

interface StoredDocument {
  namespace: string;
  collection: string;
  key: string;
  value: Binary | Uint8Array;
  codec: string;
  createdAt: string;
  updatedAt: string;
  expiresAt: string | null;
}

/** MongoDB driver for yugendb document-backed storage. */
export class MongoDbDriver implements Driver {
  readonly name = "mongodb";
  readonly options: Required<MongoDbDriverOptions>;
  private client: MongoClient | null = null;

  constructor(options: MongoDbDriverOptions | string) {
    this.options = typeof options === "string"
      ? {
          connectionString: options,
          database: "yugendb",
          storageCollection: storageCollectionName,
          createIndexesOnInitialise: true,
        }
      : {
          connectionString: options.connectionString,
          database: options.database ?? "yugendb",
          storageCollection: options.storageCollection ?? storageCollectionName,
          createIndexesOnInitialise: options.createIndexesOnInitialise ?? true,
        };
  }

  capabilities(): Capabilities {
    return mongodbCapabilities();
  }

  schema(): MongoDbDocumentSchema {
    return mongodbDocumentSchema();
  }

  async initialise(): Promise<void> {
    if (this.client !== null) {
      return;
    }

    this.client = new MongoClient(this.options.connectionString);
    await this.client.connect();

    if (this.options.createIndexesOnInitialise) {
      const collection = this.documents();
      await collection.createIndex(
        { namespace: 1, collection: 1, key: 1 },
        { unique: true, name: "idx_yugendb_identity" },
      );
      await collection.createIndex({ expiresAt: 1 }, { name: "idx_yugendb_expires_at" });
    }
  }

  async finalise(): Promise<void> {
    await this.client?.close();
    this.client = null;
  }

  async get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null> {
    const document = await this.documents().findOne({ namespace, collection, key });
    if (document === null) {
      return null;
    }

    const value = documentToStoredValue(document);
    return isExpired(value) && options.allowExpired !== true ? null : value;
  }

  async set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    options: WriteOptions,
  ): Promise<void> {
    if (options.overwrite === false && await this.exists(namespace, collection, key)) {
      throw new YugenDbError({
        code: "CONFLICT",
        message: "MongoDB driver refused to overwrite an existing value.",
      });
    }

    await this.documents().updateOne(
      { namespace, collection, key },
      { $set: storedValueToDocument(namespace, collection, key, value) },
      { upsert: true },
    );
  }

  async delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean> {
    const result = await this.documents().deleteOne({ namespace, collection, key });
    const removed = result.deletedCount > 0;

    if (options.mustExist === true && !removed) {
      throw new YugenDbError({
        code: "NOT_FOUND",
        message: "MongoDB driver could not delete a missing value.",
      });
    }

    return removed;
  }

  async exists(namespace: Namespace, collection: CollectionName, key: Key): Promise<boolean> {
    return (await this.get(namespace, collection, key, { allowExpired: false })) !== null;
  }

  async scanPrefix(
    namespace: Namespace,
    collection: CollectionName,
    options: ScanOptions,
  ): Promise<Array<[Key, StoredValue]>> {
    const prefix = options.prefix ?? "";
    const cursor = this.documents()
      .find({ namespace, collection, key: { $regex: `^${escapeRegExp(prefix)}` } })
      .sort({ key: 1 });
    const values: Array<[Key, StoredValue]> = [];

    for await (const document of cursor) {
      const value = documentToStoredValue(document);
      if (isExpired(value) && options.includeExpired !== true) {
        continue;
      }
      values.push([normaliseKey(document.key), value]);
      if (options.limit !== undefined && values.length >= options.limit) {
        break;
      }
    }

    return values;
  }

  async batch(batch: Batch): Promise<BatchResult> {
    let appliedOperations = 0;
    let deletedKeys = 0;

    for (const operation of batch.operations) {
      if (operation.kind === "set") {
        await this.set(operation.namespace, operation.collection, operation.key, operation.value, operation.options ?? {});
        appliedOperations += 1;
      } else {
        if (await this.delete(operation.namespace, operation.collection, operation.key, operation.options ?? {})) {
          deletedKeys += 1;
        }
        appliedOperations += 1;
      }
    }

    return { appliedOperations, deletedKeys };
  }

  private documents(): MongoCollection<StoredDocument> {
    if (this.client === null) {
      throw new YugenDbError({
        code: "CONNECTION_ERROR",
        message: "MongoDB driver has not been initialised.",
      });
    }
    return this.client.db(this.options.database).collection<StoredDocument>(this.options.storageCollection);
  }
}

/** Returns the capabilities reported by the MongoDB driver. */
export function mongodbCapabilities(): Capabilities {
  return {
    ...minimalCapabilities(),
    ttl: true,
    prefixScan: true,
    batchWrite: true,
    connectionPooling: true,
  };
}

/** Creates a MongoDB yugendb driver. */
export function mongodb(
  connectionString: string,
  options: Omit<MongoDbDriverOptions, "connectionString"> = {},
): MongoDbDriver {
  return new MongoDbDriver({ connectionString, ...options });
}

function storedValueToDocument(
  namespace: Namespace,
  collection: CollectionName,
  key: Key,
  value: StoredValue,
): StoredDocument {
  return {
    namespace,
    collection,
    key,
    value: new Binary(Buffer.from(value.bytes)),
    codec: value.metadata.codec,
    createdAt: value.metadata.createdAt.toISOString(),
    updatedAt: value.metadata.updatedAt.toISOString(),
    expiresAt: value.metadata.expiresAt instanceof Date ? value.metadata.expiresAt.toISOString() : null,
  };
}

function documentToStoredValue(document: StoredDocument): StoredValue {
  const raw = document.value instanceof Binary ? document.value.buffer : document.value;
  return {
    bytes: new Uint8Array(raw),
    metadata: {
      codec: document.codec,
      createdAt: new Date(document.createdAt),
      updatedAt: new Date(document.updatedAt),
      expiresAt: document.expiresAt === null ? null : new Date(document.expiresAt),
    },
  };
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
