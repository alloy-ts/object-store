import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Delete - remove object", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("to_delete1.txt", Buffer.from("temp1"));

  await store.delete("to_delete1.txt");
  await expect(store.get("to_delete1.txt")).rejects.toThrow();
});

test("Delete - deleteStream reports one result per location", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("a.txt", Buffer.from("a"));
  await store.put("b.txt", Buffer.from("b"));

  const results = await store.deleteStream(["a.txt", "missing.txt", "b.txt"]);
  expect(results).toHaveLength(3);
  expect(results[0].error).toBeUndefined();
  expect(results[1].error).toBeDefined();
  expect(results[2].error).toBeUndefined();

  // The two existing objects are gone; the missing one never existed.
  await expect(store.get("a.txt")).rejects.toThrow();
  await expect(store.get("b.txt")).rejects.toThrow();
});