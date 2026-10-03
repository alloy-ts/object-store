import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";

test("Delete - remove object", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("to_delete.txt", Buffer.from("temp"));

  await store.delete("to_delete.txt");
  await expect(store.get("to_delete.txt")).rejects.toThrow();
});
