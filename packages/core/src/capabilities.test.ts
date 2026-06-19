import { describe, expect, it } from "vitest";
import { memoryCapabilities, minimalCapabilities, sqliteCapabilities } from "./capabilities";

describe("capability helpers", () => {
  it("returns minimal capabilities", () => {
    expect(minimalCapabilities().batchWrite).toBe(false);
  });

  it("returns memory capabilities", () => {
    const capabilities = memoryCapabilities();
    expect(capabilities.ttl).toBe(true);
    expect(capabilities.prefixScan).toBe(true);
    expect(capabilities.rawSql).toBe(false);
  });

  it("returns SQLite capabilities", () => {
    const capabilities = sqliteCapabilities();
    expect(capabilities.transactions).toBe(true);
    expect(capabilities.rawSql).toBe(false);
  });
});
