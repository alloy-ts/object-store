import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("get and getResult methods", async () => {
  const store = ObjectStore.memory();
  await store.put("hello.txt", Buffer.from("hello world"));

  const buffer = await store.get("hello.txt");
  assert.equal(buffer.toString("utf8"), "hello world");

  const result = await store.getResult("hello.txt");
  assert.equal(result.meta.location, "hello.txt");
  assert.equal(result.bytes.toString("utf8"), "hello world");
});

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

  const sliceWithRange = await store.getOpts("data.txt", {
    range: { start: 2, end: 6 },
  });
  assert.equal(sliceWithRange.toString("utf8"), "2345");
});

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
