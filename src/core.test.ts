import { expect, test } from "vite-plus/test";
import { ObjectStore } from "./core.ts";

test("Core API - Put, Get, Head, List", async () => {
  const store = ObjectStore.createInMemory();

  // Put Object
  await store.put("data/file1.parquet", Buffer.from("hello world"));
  await store.put("data/file2.parquet", Buffer.from("another object content"));

  // Fetch object metadata & content
  const meta = await store.head("data/file1.parquet");
  expect(meta.location).toBe("data/file1.parquet");
  expect(meta.size).toBe(11);

  const bytes = await store.get("data/file1.parquet");
  expect(bytes.toString()).toBe("hello world");

  const getWithMeta = await store.getWithMeta("data/file1.parquet");
  expect(getWithMeta.bytes.toString()).toBe("hello world");
  expect(getWithMeta.meta.size).toBe(11);

  // List objects
  const list = await store.list("data");
  expect(list.length).toBe(2);
  const locations = list.map((item) => item.location).sort();
  expect(locations).toEqual(["data/file1.parquet", "data/file2.parquet"]);
});

test("Core API - Vectored Read (getRanges)", async () => {
  const store = ObjectStore.createInMemory();
  const content = Buffer.from("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ");
  await store.put("data/large_file", content);

  const ranges = await store.getRanges("data/large_file", [
    { start: 0, end: 10 },
    { start: 10, end: 20 },
  ]);

  expect(ranges.length).toBe(2);
  expect(ranges[0]!.toString()).toBe("0123456789");
  expect(ranges[1]!.toString()).toBe("ABCDEFGHIJ");
});

test("Core API - Copy, Rename, Delete", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("data/original.txt", Buffer.from("initial content"));

  // Copy
  await store.copy("data/original.txt", "data/copied.txt");
  const copiedData = await store.get("data/copied.txt");
  expect(copiedData.toString()).toBe("initial content");

  // Rename
  await store.rename("data/copied.txt", "data/renamed.txt");
  const renamedData = await store.get("data/renamed.txt");
  expect(renamedData.toString()).toBe("initial content");

  // Delete
  await store.delete("data/renamed.txt");
  await expect(store.get("data/renamed.txt")).rejects.toThrow();
});
