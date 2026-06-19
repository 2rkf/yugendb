/** Redis keyspace helpers for yugendb. */
export const defaultKeyPrefix = "yugendb";
export const keySeparator = ":";

function encodeComponent(value: string): string {
  return encodeURIComponent(value).replace(/%/gu, "~");
}

function decodeComponent(value: string): string {
  return decodeURIComponent(value.replace(/~/gu, "%"));
}

/** Composes the Redis key for a yugendb namespace, collection, and key. */
export function redisKey(prefix: string, namespace: string, collection: string, key: string): string {
  return [prefix, encodeComponent(namespace), encodeComponent(collection), encodeComponent(key)].join(keySeparator);
}

/** Composes a Redis prefix for scanning a yugendb collection. */
export function redisPrefix(prefix: string, namespace: string, collection: string, keyPrefix = ""): string {
  return [prefix, encodeComponent(namespace), encodeComponent(collection), encodeComponent(keyPrefix)].join(keySeparator);
}

/** Decodes a Redis key back to the original yugendb key when it matches the prefix. */
export function decodeRedisKey(prefix: string, namespace: string, collection: string, key: string): string | null {
  const start = redisPrefix(prefix, namespace, collection, "");
  if (!key.startsWith(start)) return null;
  return decodeComponent(key.slice(start.length));
}

/** Keyspace settings used by the Redis driver. */
export interface RedisKeyspace {
  readonly keyPrefix: string;
}

/** Creates Redis keyspace settings. */
export function redisKeyspace(keyPrefix = defaultKeyPrefix): RedisKeyspace {
  return { keyPrefix };
}

/** Composes the full storage key used by the Redis driver. */
export function composeStorageKey(prefix: string, namespace: string, collection: string, key: string): string {
  return redisKey(prefix, namespace, collection, key);
}

/** Composes the collection prefix used by Redis scans. */
export function composeCollectionPrefix(prefix: string, namespace: string, collection: string): string {
  return redisPrefix(prefix, namespace, collection, "");
}
