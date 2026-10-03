import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

// @ts-ignore
import { ObjectStore, parseUrl } from "../../index.js";

test("ObjectStore.memory factory", async () => {
  const store = ObjectStore.memory();
  await store.put("factory_test.txt", Buffer.from("memory factory"));
  const data = await store.get("factory_test.txt");
  assert.equal(data.toString("utf8"), "memory factory");
});

test("ObjectStore.local factory", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "obj-store-factory-"));
  try {
    const store = ObjectStore.local(tmpDir);
    await store.put("local_test.txt", Buffer.from("local factory"));
    const data = await store.get("local_test.txt");
    assert.equal(data.toString("utf8"), "local factory");
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("parseUrl standalone and ObjectStore.parseUrl", async () => {
  const res = parseUrl("memory://");
  assert.ok(res.store);
  assert.equal(res.path, "");

  const res2 = ObjectStore.parseUrl("memory://");
  assert.ok(res2.store);
  assert.equal(res2.path, "");
});
