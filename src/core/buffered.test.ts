import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { BufReader, BufWriter, ObjectStore } from "../../index.js";

test("BufWriter and BufReader buffered IO", async () => {
  const store = ObjectStore.memory();

  // Write buffered chunks
  const writer = new BufWriter(store, "buffered.txt");
  await writer.write(Buffer.from("Hello "));
  await writer.write(Buffer.from("Buffered "));
  await writer.write(Buffer.from("World!"));
  await writer.finish();

  const data = await store.get("buffered.txt");
  assert.equal(data.toString("utf8"), "Hello Buffered World!");

  // Read buffered chunks
  const reader = await BufReader.open(store, "buffered.txt", 1024);
  const chunk1 = await reader.read(5);
  assert.equal(chunk1.toString("utf8"), "Hello");

  await reader.seek(15);
  const chunk2 = await reader.read(6);
  assert.equal(chunk2.toString("utf8"), "World!");
});
