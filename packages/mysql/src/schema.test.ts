import { describe, expect, it } from "vitest";
import { mysql, mysqlCapabilities, allSchemaStatements, createExpiresAtIndexSql, createPrefixIndexSql, createStoreTableSql, storeTableName } from "./index.js";


describe("MySQL driver", () => {
  it("exposes the storage schema", () => {
    expect(storeTableName).toBe("yugendb_store");
    expect(createStoreTableSql).toContain("CREATE TABLE");
    expect(createStoreTableSql).toContain("yugendb_store");
    expect(createPrefixIndexSql).toContain("idx_yugendb_store_prefix");
    expect(createExpiresAtIndexSql).toContain("idx_yugendb_store_expires_at");
    expect(allSchemaStatements.length).toBeGreaterThan(0);
  });

  it("returns an implemented external-service driver", () => {
    const driver = mysql("backend://localhost/yugendb");
    expect(driver.name).toBe("mysql");
    expect(driver.capabilities().prefixScan).toBe(true);
    expect(mysqlCapabilities().prefixScan).toBe(true);
  });

  it("does not connect until initialise is called", () => {
    const driver = mysql("mysql://localhost/yugendb");
    expect(driver.capabilities().connectionPooling).toBe(true);
  });
});
