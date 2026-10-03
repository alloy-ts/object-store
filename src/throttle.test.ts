import { expect, test } from "vite-plus/test";
import { InMemory, ThrottledStore } from "../index.js";

test("ThrottledStore - wraps InMemory and round-trips through ObjectStore", async () => {
  const inner = new InMemory();
  const throttled = ThrottledStore.new(inner, {
    waitPutPerCall: 0,
    waitGetPerCall: 0,
  });

  // The throttled store is surfaced as a regular ObjectStore.
  const store = throttled.asObjectStore();
  await store.put("a.txt", Buffer.from("throttled"));
  const data = await store.get("a.txt");
  expect(data.toString()).toBe("throttled");

  // Config is observable and mutable.
  const before = throttled.getConfig();
  expect(before.waitPutPerCall).toBe(0);

  throttled.configure({ waitPutPerCall: 50 });
  const after = throttled.getConfig();
  expect(after.waitPutPerCall).toBe(50);
});

test("ThrottledStore - exposes the low-level MultipartStore too", async () => {
  const inner = new InMemory();
  const throttled = ThrottledStore.new(inner);
  const mp = throttled.asMultipartStore();

  const id = await mp.createMultipart("big.bin");
  const p0 = await mp.putPart("big.bin", id, 0, Buffer.from("foo"));
  const p1 = await mp.putPart("big.bin", id, 1, Buffer.from("bar"));
  await mp.completeMultipart("big.bin", id, [p0, p1]);

  // Verify the data landed in the (shared) inner store.
  const data = await inner.get("big.bin");
  expect(data.toString()).toBe("foobar");
});
