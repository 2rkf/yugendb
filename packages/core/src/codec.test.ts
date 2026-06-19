import { describe, expect, it } from "vitest";
import { JsonCodec } from "./codec";
import { YugenDbError } from "./errors";

describe("JsonCodec", () => {
  it("serialises and deserialises typed values", () => {
    const codec = new JsonCodec();
    const bytes = codec.serialise({ id: "user_001", enabled: true });
    const value = codec.deserialise<{ id: string; enabled: boolean }>(bytes);

    expect(value).toEqual({ id: "user_001", enabled: true });
  });

  it("maps serialisation failures", () => {
    const codec = new JsonCodec();
    expect(() => codec.serialise({ value: 1n })).toThrow(YugenDbError);
  });

  it("maps deserialisation failures", () => {
    const codec = new JsonCodec();
    expect(() => codec.deserialise(new TextEncoder().encode("{"))).toThrow(YugenDbError);
  });
});
