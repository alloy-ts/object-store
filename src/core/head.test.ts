import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Head - fetch object metadata and headOpts", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("sample.txt", Buffer.from("sample content"));

  const meta = await store.head("sample.txt");
  expect(meta.location).toBe("sample.txt");
  expect(meta.size).toBe(14);

  const metaOpts = await store.headOpts("sample.txt");
  expect(metaOpts.location).toBe("sample.txt");
  expect(metaOpts.size).toBe(14);
});
