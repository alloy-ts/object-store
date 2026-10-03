import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("list and listWithOffset methods", async () => {
  const store = ObjectStore.memory();
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/b.txt", Buffer.from("b"));
  await store.put("dir/c.txt", Buffer.from("c"));

  const items = await store.list("dir");
  assert.equal(items.length, 3);
  const locations = items.map((i: any) => i.location).sort();
  assert.deepEqual(locations, ["dir/a.txt", "dir/b.txt", "dir/c.txt"]);

  const itemsWithOffset = await store.listWithOffset("dir", "dir/a.txt");
  assert.ok(itemsWithOffset.length >= 1);
});
