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
