import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";

test("Rename - move object and renameOpts", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("source.txt", Buffer.from("content"));

  await store.rename("source.txt", "dest.txt");
  const destData = await store.get("dest.txt");
  expect(destData.toString()).toBe("content");

  await store.renameOpts("dest.txt", "dest2.txt");
  const dest2Data = await store.get("dest2.txt");
  expect(dest2Data.toString()).toBe("content");

  await expect(store.get("source.txt")).rejects.toThrow();
});
