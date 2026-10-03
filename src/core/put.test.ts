import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("put method", async () => {
  const store = ObjectStore.memory();
  const res = await store.put("item.txt", Buffer.from("item content"));
  assert.ok(res);

  const data = await store.get("item.txt");
  assert.equal(data.toString("utf8"), "item content");
});

test("putOpts method with tags and mode", async () => {
  const store = ObjectStore.memory();
  const res = await store.putOpts("opts.txt", Buffer.from("opts content"), {
    mode: "overwrite",
    tags: { env: "test", project: "napi" },
  });
  assert.ok(res);

  const data = await store.get("opts.txt");
  assert.equal(data.toString("utf8"), "opts content");
});
