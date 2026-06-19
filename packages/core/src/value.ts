import { YugenDbError } from "./errors";

/** Serialised value bytes stored by a yugendb driver. */
export type ValueBytes = Uint8Array;

/** Metadata stored alongside each serialised value. */
export interface ValueMetadata {
  /** Codec used to serialise the value. */
  readonly codec: string;
  /** Time the value was first created. */
  readonly createdAt: Date;
  /** Time the value was last updated. */
  readonly updatedAt: Date;
  /** Optional expiry time; null means the value does not expire. */
  readonly expiresAt?: Date | null;
}

/** Serialised value and metadata returned by drivers. */
export interface StoredValue {
  /** Value bytes produced by the configured codec. */
  readonly bytes: ValueBytes;
  /** Metadata stored with the value. */
  readonly metadata: ValueMetadata;
}

/** Creates value metadata, optionally applying a time-to-live in milliseconds. */
export function createValueMetadata(
  codec: string,
  ttlMs?: number,
  now: Date = new Date(),
): ValueMetadata {
  if (ttlMs !== undefined) {
    if (!Number.isFinite(ttlMs) || ttlMs < 0) {
      throw new YugenDbError({
        code: "INVALID_VALUE",
        message: "Invalid value: ttlMs must be a non-negative finite number.",
      });
    }

    return {
      codec,
      createdAt: now,
      updatedAt: now,
      expiresAt: new Date(now.getTime() + ttlMs),
    };
  }

  return {
    codec,
    createdAt: now,
    updatedAt: now,
    expiresAt: null,
  };
}

/** Returns true when a stored value has expired at the provided time. */
export function isExpired(value: StoredValue, now: Date = new Date()): boolean {
  const expiresAt = value.metadata.expiresAt;
  return expiresAt instanceof Date && expiresAt.getTime() <= now.getTime();
}
