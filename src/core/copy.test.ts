import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";

test("Copy - duplicate object", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("orig.txt", Buffer.from("data"));

  await store.copy("orig.txt", "copy.txt");
  const copyData = await store.get("copy.txt");
  expect(copyData.toString()).toBe("data");
});
