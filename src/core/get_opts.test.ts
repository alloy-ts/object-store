import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("getOpts method with range and preconditions", async () => {
  const store = ObjectStore.memory();
  await store.put("data.txt", Buffer.from("0123456789"));

  const slice = await store.getOpts("data.txt", {
    rangeStart: 2,
    rangeEnd: 6,
    ifModifiedSince: new Date(0).toISOString(),
    head: false,
  });
  assert.equal(slice.toString("utf8"), "2345");
});
