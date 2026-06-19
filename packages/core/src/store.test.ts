import { describe, expect, it } from "vitest";
import type { Batch, BatchResult } from "./batch";
import { memoryCapabilities, type Capabilities } from "./capabilities";
import type { Driver } from "./driver";
import type { CollectionName, Key, Namespace } from "./key";
import type { DeleteOptions, ReadOptions, ScanOptions, WriteOptions } from "./options";
import { createStore } from "./store";
import type { StoredValue } from "./value";

class MockDriver implements Driver {
  readonly name = "mock";
  initialiseCount = 0;
  finaliseCount = 0;
  lastCollection: CollectionName | null = null;
  private readonly values = new Map<string, StoredValue>();

  capabilities(): Capabilities {
    return memoryCapabilities();
  }

  async initialise(): Promise<void> {
    this.initialiseCount += 1;
  }

  async finalise(): Promise<void> {
    this.finaliseCount += 1;
  }

  async get(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    _options: ReadOptions,
  ): Promise<StoredValue | null> {
    return this.values.get(this.id(namespace, collection, key)) ?? null;
  }

  async set(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    value: StoredValue,
    _options: WriteOptions,
  ): Promise<void> {
    this.lastCollection = collection;
    this.values.set(this.id(namespace, collection, key), value);
  }

  async delete(
    namespace: Namespace,
    collection: CollectionName,
    key: Key,
    _options: DeleteOptions,
  ): Promise<boolean> {
    return this.values.delete(this.id(namespace, collection, key));
  }

  async exists(namespace: Namespace, collection: CollectionName, key: Key): Promise<boolean> {
    return this.values.has(this.id(namespace, collection, key));
  }

  async scanPrefix(
    namespace: Namespace,
    collection: CollectionName,
    options: ScanOptions,
  ): Promise<Array<[Key, StoredValue]>> {
    const prefix = options.prefix ?? "";
    const base = `${namespace}/${collection}/`;
    const rows: Array<[Key, StoredValue]> = [];

    for (const [id, value] of this.values.entries()) {
      if (!id.startsWith(base)) {
        continue;
      }

      const key = id.slice(base.length) as Key;
      if (key.startsWith(prefix)) {
        rows.push([key, value]);
      }
    }

    return rows.sort(([left], [right]) => left.localeCompare(right));
  }

  async batch(batch: Batch): Promise<BatchResult> {
    return { appliedOperations: batch.operations.length };
  }

  private id(namespace: Namespace, collection: CollectionName, key: Key): string {
    return `${namespace}/${collection}/${key}`;
  }
}

describe("Store", () => {
  it("createStore calls driver.initialise", async () => {
    const driver = new MockDriver();
    await createStore({ driver, namespace: "tests" });
    expect(driver.initialiseCount).toBe(1);
  });

  it("returns null when the driver returns no stored value", async () => {
    const driver = new MockDriver();
    const store = await createStore({ driver, namespace: "tests" });

    await expect(store.get<{ id: string }>("missing")).resolves.toBeNull();
  });

  it("stores and loads typed values", async () => {
    const driver = new MockDriver();
    const store = await createStore({ driver, namespace: "tests" });

    await store.set("user_001", { id: "user_001" });
    await expect(store.get<{ id: string }>("user_001")).resolves.toEqual({ id: "user_001" });
  });

  it("uses the given collection name", async () => {
    const driver = new MockDriver();
    const store = await createStore({ driver, namespace: "tests" });
    const users = store.collection<{ id: string }>("users");

    await users.set("user_001", { id: "user_001" });

    expect(driver.lastCollection).toBe("users");
  });
});
