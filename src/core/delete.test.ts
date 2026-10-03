import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("delete and deleteOpts methods", async () => {
  const store = ObjectStore.memory();
  await store.put("to_delete.txt", Buffer.from("temp"));

  await store.delete("to_delete.txt");
  await assert.rejects(async () => {
    await store.get("to_delete.txt");
  });

  await store.put("to_delete2.txt", Buffer.from("temp2"));
  await store.deleteOpts("to_delete2.txt", {});
  await assert.rejects(async () => {
    await store.get("to_delete2.txt");
  });
});
