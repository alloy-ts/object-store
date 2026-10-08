import { expect, test } from "vite-plus/test";
import { InMemory, MultipartStore } from "../dist/index.js";

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

test("MultipartStore - createMultipartOpts forwards attributes to the object", async () => {
  const store = new InMemory();
  const mp = MultipartStore.fromInMemory(store);

  const id = await mp.createMultipartOpts("attrs.bin", {
    tags: { team: "storage" },
    attributes: { "cache-control": "max-age=60" },
  });
  const p0 = await mp.putPart("attrs.bin", id, 0, Buffer.from("data"));
  await mp.completeMultipart("attrs.bin", id, [p0]);

  const res = await store.asObjectStore().getWithMeta("attrs.bin");
  expect(res.attributes["cache-control"]).toBe("max-age=60");
  expect((await store.asObjectStore().get("attrs.bin")).toString()).toBe("data");
});
