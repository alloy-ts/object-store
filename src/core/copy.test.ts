import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Copy - duplicate object and copyOpts", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("orig.txt", Buffer.from("data"));

  await store.copy("orig.txt", "copy.txt");
  const copyData = await store.get("copy.txt");
  expect(copyData.toString()).toBe("data");

  await store.copyOpts("orig.txt", "copy2.txt");
  const copy2Data = await store.get("copy2.txt");
  expect(copy2Data.toString()).toBe("data");
});
