import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Get - get, getWithMeta, getOpts", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("hello.txt", Buffer.from("Hello World!"));

  const data = await store.get("hello.txt");
  expect(data.toString()).toBe("Hello World!");

  const res = await store.getWithMeta("hello.txt");
  expect(res.bytes.toString()).toBe("Hello World!");
  expect(res.meta.location).toBe("hello.txt");
  expect(res.meta.size).toBe(12);

  const rangeData = await store.get("hello.txt", {
    range: { start: 0, end: 5 },
  });
  expect(rangeData.toString()).toBe("Hello");

  const rangeDataOpts = await store.getOpts("hello.txt", {
    range: { start: 0, end: 5 },
  });
  expect(rangeDataOpts.toString()).toBe("Hello");
});
