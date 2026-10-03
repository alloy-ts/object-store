import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";

test("Delete - remove object and deleteOpts", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("to_delete1.txt", Buffer.from("temp1"));
  await store.put("to_delete2.txt", Buffer.from("temp2"));

  await store.delete("to_delete1.txt");
  await expect(store.get("to_delete1.txt")).rejects.toThrow();

  await store.deleteOpts("to_delete2.txt");
  await expect(store.get("to_delete2.txt")).rejects.toThrow();
});
