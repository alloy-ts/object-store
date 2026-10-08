import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Put - put and putOpts", async () => {
  const store = ObjectStore.createInMemory();

  const res1 = await store.put("file.txt", Buffer.from("content"));
  expect(res1).toBeDefined();

  const data1 = await store.get("file.txt");
  expect(data1.toString()).toBe("content");

  const res2 = await store.put("file.txt", Buffer.from("new content"), {
    modeOverwrite: true,
  });
  expect(res2).toBeDefined();

  const data2 = await store.get("file.txt");
  expect(data2.toString()).toBe("new content");
});

test("Put - putOpts modeCreate only writes when the key is absent", async () => {
  const store = ObjectStore.createInMemory();

  const created = await store.put("fresh.txt", Buffer.from("first"), {
    modeCreate: true,
  });
  expect(created).toBeDefined();

  // A second create on an existing key must be rejected.
  await expect(
    store.put("fresh.txt", Buffer.from("second"), { modeCreate: true }),
  ).rejects.toBeDefined();

  // The original content is preserved.
  expect((await store.get("fresh.txt")).toString()).toBe("first");
});

test("Put - putOpts modeUpdate requires the matching e_tag", async () => {
  const store = ObjectStore.createInMemory();
  const first = await store.put("v.txt", Buffer.from("v1"));
  const eTag = first.eTag!; // InMemory assigns an e_tag on write

  // A matching e_tag allows the update.
  await store.put("v.txt", Buffer.from("v2"), { modeUpdate: { eTag } });
  expect((await store.get("v.txt")).toString()).toBe("v2");

  // A non-matching e_tag must be rejected.
  await expect(
    store.put("v.txt", Buffer.from("v3"), {
      modeUpdate: { eTag: "does-not-match" },
    }),
  ).rejects.toBeDefined();
});

test("Put - putMultipart streams parts and completes into one object", async () => {
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipart("big.bin");
  expect(upload).toBeDefined();

  await upload.putPart(Buffer.from("hello "));
  await upload.putPart(Buffer.from("world"));

  const res = await upload.complete();
  expect(res.eTag).toBeDefined();

  const data = await store.get("big.bin");
  expect(data.toString()).toBe("hello world");
});

test("Put - putMultipart accepts PutPayload parts", async () => {
  const { PutPayload } = await import("../../dist/index.js");
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipart("payloads.bin");
  await upload.putPart(PutPayload.fromBytes(Buffer.from("foo")));
  await upload.putPart(PutPayload.fromString("bar"));
  await upload.complete();

  expect((await store.get("payloads.bin")).toString()).toBe("foobar");
});

test("Put - putMultipart abort discards the in-progress upload", async () => {
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipart("aborted.bin");
  await upload.putPart(Buffer.from("partial"));
  await upload.abort();

  // The object must not exist after abort.
  await expect(store.get("aborted.bin")).rejects.toBeDefined();
});
