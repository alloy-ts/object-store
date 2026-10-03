import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("getRanges method", async () => {
  const store = ObjectStore.memory();
  await store.put("vectored.txt", Buffer.from("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));

  const ranges = await store.getRanges("vectored.txt", [
    { start: 0, end: 3 },
    { start: 10, end: 13 },
  ]);

  assert.equal(ranges.length, 2);
  assert.equal(ranges[0]!.toString("utf8"), "ABC");
  assert.equal(ranges[1]!.toString("utf8"), "KLM");
});
