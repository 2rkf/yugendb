import { YugenDbError } from "./errors";
import type { ValueBytes } from "./value";

/** Converts application values to and from bytes for storage. */
export interface Codec {
  /** Stable codec name stored in value metadata. */
  readonly name: string;
  /** Serialises an application value into bytes. */
  serialise(value: unknown): ValueBytes;
  /** Deserialises bytes into an application value. */
  deserialise<T>(bytes: ValueBytes): T;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** JSON codec for typed values that can be represented as JSON. */
export class JsonCodec implements Codec {
  readonly name = "json";

  serialise(value: unknown): ValueBytes {
    try {
      const text = JSON.stringify(value);

      if (text === undefined) {
        throw new YugenDbError({
          code: "SERIALISATION_ERROR",
          message: "Failed to serialise value: JSON.stringify returned undefined.",
        });
      }

      return encoder.encode(text);
    } catch (error) {
      if (error instanceof YugenDbError) {
        throw error;
      }

      throw new YugenDbError({
        code: "SERIALISATION_ERROR",
        message: "Failed to serialise value as JSON.",
        cause: error,
      });
    }
  }

  deserialise<T>(bytes: ValueBytes): T {
    try {
      return JSON.parse(decoder.decode(bytes)) as T;
    } catch (error) {
      throw new YugenDbError({
        code: "DESERIALISATION_ERROR",
        message: "Failed to deserialise value from JSON.",
        cause: error,
      });
    }
  }
}
