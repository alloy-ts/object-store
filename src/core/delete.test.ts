import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("delete method", async () => {
  const store = ObjectStore.memory();
  await store.put("to_delete.txt", Buffer.from("temp"));

  await store.delete("to_delete.txt");
  await assert.rejects(async () => {
    await store.get("to_delete.txt");
  });
});
