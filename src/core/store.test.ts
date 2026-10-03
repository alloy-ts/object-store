import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";
import fs from "node:fs";
import path from "node:path";
import os = require("node:os");

test("Store - Factory methods (in-memory, local, parseUrl)", async () => {
  const memStore = ObjectStore.createInMemory();
  expect(memStore).toBeDefined();

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "store-test-"));
  try {
    const localStore = ObjectStore.createLocal(tmpDir);
    expect(localStore).toBeDefined();
    await localStore.put("test.txt", Buffer.from("local"));
    const data = await localStore.get("test.txt");
    expect(data.toString()).toBe("local");
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }

  const memoryUrlStore = ObjectStore.parseUrl("memory://");
  expect(memoryUrlStore).toBeDefined();
});
