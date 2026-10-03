import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("list method", async () => {
  const store = ObjectStore.memory();
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/b.txt", Buffer.from("b"));

  const items = await store.list("dir");
  assert.equal(items.length, 2);
  const locations = items.map((i: any) => i.location).sort();
  assert.deepEqual(locations, ["dir/a.txt", "dir/b.txt"]);
});
