import { describe, expect, it } from "vitest";
import { identityIndexFields, mongodb, mongodbCapabilities, storageCollectionName, storedDocumentFields } from "./index.js";

describe("MongoDB driver", () => {
  it("exposes document mapping fields", () => {
    expect(storageCollectionName).toBe("yugendb_store");
    expect(identityIndexFields()).toEqual(["namespace", "collection", "key"]);
    expect(storedDocumentFields).toContain("value");
    expect(storedDocumentFields).toContain("expiresAt");
  });

  it("returns an implemented external-service driver", () => {
    const driver = mongodb("mongodb://localhost:27017");
    expect(driver.name).toBe("mongodb");
    expect(driver.capabilities().documentQuery).toBe(false);
    expect(mongodbCapabilities().prefixScan).toBe(true);
  });

  it("does not connect until initialise is called", () => {
    const driver = mongodb("mongodb://localhost:27017");
    expect(driver.capabilities().connectionPooling).toBe(true);
  });
});
