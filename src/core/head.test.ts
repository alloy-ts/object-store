import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";

test("Head - fetch object metadata", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("sample.txt", Buffer.from("sample content"));

  const meta = await store.head("sample.txt");
  expect(meta.location).toBe("sample.txt");
  expect(meta.size).toBe(14);
});
