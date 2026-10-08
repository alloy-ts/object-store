import { expect, test } from "vite-plus/test";
import { ObjectStore, LimitStore } from "../index.js";

test("LimitStore - wraps a store and round-trips through ObjectStore", async () => {
  const inner = ObjectStore.createInMemory();
  const limited = LimitStore.new(inner, 2);

  // The limited store is surfaced as a regular ObjectStore.
  const store = limited.asObjectStore();
  await store.put("a.txt", Buffer.from("limited"));
  const data = await store.get("a.txt");
  expect(data.toString()).toBe("limited");
});

test("LimitStore - rejects a max_requests of zero", () => {
  const inner = ObjectStore.createInMemory();
  expect(() => LimitStore.new(inner, 0)).toThrow();
});

test("LimitStore - bounded concurrency still completes every operation", async () => {
  const inner = ObjectStore.createInMemory();
  // A very small limit must not deadlock or drop work.
  const limited = LimitStore.new(inner, 1);
  const store = limited.asObjectStore();

  const keys = Array.from({ length: 20 }, (_, i) => `key-${i}.txt`);
  await Promise.all(
    keys.map((k, i) => store.put(k, Buffer.from(`value-${i}`))),
  );

  for (let i = 0; i < keys.length; i++) {
    const data = await store.get(keys[i]);
    expect(data.toString()).toBe(`value-${i}`);
  }
});

test("LimitStore - shares the backing store with the inner store", async () => {
  const inner = ObjectStore.createInMemory();
  const limited = LimitStore.new(inner, 4);

  // Write through the limited store...
  const limitedStore = limited.asObjectStore();
  await limitedStore.put("shared.txt", Buffer.from("via-limit"));

  // ...and read it back directly from the inner store.
  const data = await inner.get("shared.txt");
  expect(data.toString()).toBe("via-limit");
});
