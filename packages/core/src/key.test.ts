import { describe, expect, it } from "vitest";
import { YugenDbError } from "./errors";
import { normaliseCollectionName, normaliseKey, normaliseNamespace } from "./key";

describe("key model", () => {
  it("accepts valid namespace, collection, and key values", () => {
    expect(normaliseNamespace("app")).toBe("app");
    expect(normaliseCollectionName("users")).toBe("users");
    expect(normaliseKey("user_001")).toBe("user_001");
  });

  it("rejects empty namespaces", () => {
    expect(() => normaliseNamespace("  ")).toThrow(YugenDbError);
  });

  it("rejects keys with control characters", () => {
    expect(() => normaliseKey("bad\nkey")).toThrow(YugenDbError);
  });
});
