import { describe, expect, it } from "vitest";
import { postgres, postgresCapabilities, allSchemaStatements, createExpiresAtIndexSql, createPrefixIndexSql, createStoreTableSql, storeTableName } from "./index.js";


describe("PostgreSQL driver", () => {
  it("exposes the storage schema", () => {
    expect(storeTableName).toBe("yugendb_store");
    expect(createStoreTableSql).toContain("CREATE TABLE");
    expect(createStoreTableSql).toContain("yugendb_store");
    expect(createPrefixIndexSql).toContain("idx_yugendb_store_prefix");
    expect(createExpiresAtIndexSql).toContain("idx_yugendb_store_expires_at");
    expect(allSchemaStatements).toHaveLength(3);
  });

  it("returns an implemented external-service driver", () => {
    const driver = postgres("backend://localhost/yugendb");
    expect(driver.name).toBe("postgres");
    expect(driver.capabilities().prefixScan).toBe(true);
    expect(postgresCapabilities().prefixScan).toBe(true);
  });

  it("does not connect until initialise is called", () => {
    const driver = postgres("postgres://localhost/yugendb");
    expect(driver.capabilities().connectionPooling).toBe(true);
  });
});
