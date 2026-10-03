import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("head method", async () => {
  const store = ObjectStore.memory();
  await store.put("meta.txt", Buffer.from("metadata test"));

  const meta = await store.head("meta.txt");
  assert.equal(meta.location, "meta.txt");
  assert.equal(meta.size, 13);
  assert.ok(meta.lastModified);
});
