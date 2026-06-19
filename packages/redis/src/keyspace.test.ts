import { describe, expect, it } from "vitest";
import { composeCollectionPrefix, composeStorageKey, defaultKeyPrefix, redis, redisCapabilities } from "./index.js";

describe("Redis driver", () => {
  it("composes deterministic internal keys", () => {
    expect(defaultKeyPrefix).toBe("yugendb");
    expect(composeStorageKey("yugendb", "app", "users", "001")).toBe("yugendb:app:users:001");
    expect(composeCollectionPrefix("yugendb", "app", "users")).toBe("yugendb:app:users:");
  });

  it("returns an implemented external-service driver", () => {
    const driver = redis("target");
    expect(driver.name).toBe("redis");
    expect(driver.capabilities().prefixScan).toBe(true);
    expect(redisCapabilities().prefixScan).toBe(true);
  });

  it("does not connect until initialise is called", () => {
    const driver = redis("redis://localhost:6379");
    expect(driver.capabilities().ttl).toBe(true);
  });
});
