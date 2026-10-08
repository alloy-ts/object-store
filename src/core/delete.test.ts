import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Delete - remove object", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("to_delete1.txt", Buffer.from("temp1"));

  await store.delete("to_delete1.txt");
  await expect(store.get("to_delete1.txt")).rejects.toThrow();
});

test("Delete - deleteStream reports one result per location, in order", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("a.txt", Buffer.from("a"));
  await store.put("b.txt", Buffer.from("b"));

  const results = await store.deleteStream(["a.txt", "missing.txt", "b.txt"]);
  expect(results).toHaveLength(3);
  expect(results.map((r) => r.path)).toEqual(["a.txt", "missing.txt", "b.txt"]);
  // Whether deleting an absent key errors is backend-specific, and `InMemory`
  // treats it as a success — only the two real objects are asserted as gone.
  expect(results[0].error).toBeUndefined();
  expect(results[2].error).toBeUndefined();

  await expect(store.get("a.txt")).rejects.toThrow();
  await expect(store.get("b.txt")).rejects.toThrow();
});