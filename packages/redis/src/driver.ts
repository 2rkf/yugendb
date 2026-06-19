import { Buffer } from "node:buffer";
import { createClient, type RedisClientType } from "redis";

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

import { decodeRedisKey, defaultKeyPrefix, redisKey, redisKeyspace, redisPrefix, type RedisKeyspace } from "./keyspace.js";

/** Options used to connect the Redis driver. */
export interface RedisDriverOptions {
  readonly connectionString: string;
  readonly keyPrefix?: string;
}

interface RedisEnvelope {
  value: string;
  codec: string;
  createdAt: string;
  updatedAt: string;
  expiresAt: string | null;
}

/** Redis driver for yugendb key-value storage. */
export class RedisDriver implements Driver {
  readonly name = "redis";
  readonly options: Required<RedisDriverOptions>;
  private client: RedisClientType | null = null;

  constructor(options: RedisDriverOptions | string) {
    this.options = typeof options === "string"
      ? { connectionString: options, keyPrefix: defaultKeyPrefix }
      : { connectionString: options.connectionString, keyPrefix: options.keyPrefix ?? defaultKeyPrefix };
  }

  capabilities(): Capabilities {
    return redisCapabilities();
  }

  keyspace(): RedisKeyspace {
    return redisKeyspace(this.options.keyPrefix);
  }

  async initialise(): Promise<void> {
    if (this.client !== null) {
      return;
    }

    this.client = createClient({ url: this.options.connectionString }) as RedisClientType;
    this.client.on("error", () => undefined);
    await this.client.connect();
  }

  async finalise(): Promise<void> {
    await this.client?.quit();
    this.client = null;
  }

  async get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: ReadOptions,
  ): Promise<StoredValue | null> {
    const raw = await this.db().get(this.storageKey(namespace, collection, key));
    if (raw === null) {
      return null;
    }

    const value = envelopeToStoredValue(JSON.parse(raw) as RedisEnvelope);
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
        message: "Redis driver refused to overwrite an existing value.",
      });
    }

    const storageKey = this.storageKey(namespace, collection, key);
    await this.db().set(storageKey, JSON.stringify(storedValueToEnvelope(value)));
    if (value.metadata.expiresAt instanceof Date) {
      await this.db().pExpireAt(storageKey, value.metadata.expiresAt.getTime());
    }
  }

  async delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    options: DeleteOptions,
  ): Promise<boolean> {
    const removed = await this.db().del(this.storageKey(namespace, collection, key));
    if (options.mustExist === true && removed === 0) {
      throw new YugenDbError({
        code: "NOT_FOUND",
        message: "Redis driver could not delete a missing value.",
      });
    }
    return removed > 0;
  }

  async exists(namespace: Namespace, collection: CollectionName, key: Key): Promise<boolean> {
    return (await this.get(namespace, collection, key, { allowExpired: false })) !== null;
  }

  async scanPrefix(
    namespace: Namespace,
    collection: CollectionName,
    options: ScanOptions,
  ): Promise<Array<[Key, StoredValue]>> {
    const pattern = `${redisPrefix(this.options.keyPrefix, namespace, collection, options.prefix ?? "")}*`;
    const keys: string[] = [];

    for await (const key of this.db().scanIterator({ MATCH: pattern })) {
      keys.push(String(key));
    }

    keys.sort();
    const values: Array<[Key, StoredValue]> = [];

    for (const storageKey of keys) {
      const key = decodeRedisKey(this.options.keyPrefix, namespace, collection, storageKey);
      if (key === null) {
        continue;
      }

      const raw = await this.db().get(storageKey);
      if (raw === null) {
        continue;
      }

      const value = envelopeToStoredValue(JSON.parse(raw) as RedisEnvelope);
      if (isExpired(value) && options.includeExpired !== true) {
        continue;
      }

      values.push([normaliseKey(key), value]);
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

  private storageKey(namespace: Namespace, collection: CollectionName, key: Key): string {
    return redisKey(this.options.keyPrefix, namespace, collection, key);
  }

  private db(): RedisClientType {
    if (this.client === null) {
      throw new YugenDbError({ code: "CONNECTION_ERROR", message: "Redis driver has not been initialised." });
    }
    return this.client;
  }
}

/** Returns the capabilities reported by the Redis driver. */
export function redisCapabilities(): Capabilities {
  return {
    ...minimalCapabilities(),
    ttl: true,
    prefixScan: true,
    batchWrite: true,
  };
}

/** Creates a Redis yugendb driver. */
export function redis(
  connectionString: string,
  options: Omit<RedisDriverOptions, "connectionString"> = {},
): RedisDriver {
  return new RedisDriver({ connectionString, ...options });
}

function storedValueToEnvelope(value: StoredValue): RedisEnvelope {
  return {
    value: Buffer.from(value.bytes).toString("base64"),
    codec: value.metadata.codec,
    createdAt: value.metadata.createdAt.toISOString(),
    updatedAt: value.metadata.updatedAt.toISOString(),
    expiresAt: value.metadata.expiresAt instanceof Date ? value.metadata.expiresAt.toISOString() : null,
  };
}

function envelopeToStoredValue(envelope: RedisEnvelope): StoredValue {
  return {
    bytes: new Uint8Array(Buffer.from(envelope.value, "base64")),
    metadata: {
      codec: envelope.codec,
      createdAt: new Date(envelope.createdAt),
      updatedAt: new Date(envelope.updatedAt),
      expiresAt: envelope.expiresAt === null ? null : new Date(envelope.expiresAt),
    },
  };
}
