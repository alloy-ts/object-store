import { expect, test } from "vite-plus/test";
import { InMemory, MultipartStore } from "../index.js";

test("MultipartStore - create / put_part / complete round-trip", async () => {
  const store = new InMemory();
  const mp = MultipartStore.fromInMemory(store);

  const id = await mp.createMultipart("big.bin");

  const p0 = await mp.putPart("big.bin", id, 0, Buffer.from("hello "));
  const p1 = await mp.putPart("big.bin", id, 1, Buffer.from("world"));

  const res = await mp.completeMultipart("big.bin", id, [p0, p1]);
  expect(res.eTag).toBeDefined();

  const data = await store.asObjectStore().get("big.bin");
  expect(data.toString()).toBe("hello world");
});

test("MultipartStore - abort discards the upload", async () => {
  const store = new InMemory();
  const mp = MultipartStore.fromInMemory(store);

  const id = await mp.createMultipart("aborted.bin");
  await mp.putPart("aborted.bin", id, 0, Buffer.from("partial"));

  await mp.abortMultipart("aborted.bin", id);

  // Completing after abort should fail / the object should not exist.
  await expect(store.asObjectStore().get("aborted.bin")).rejects.toBeDefined();
});
