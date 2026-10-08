import { expect, test } from "vite-plus/test";
import { ObjectStore, InMemory, PutPayload, PutPayloadMut } from "../../index.js";

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

test("PutPayload - constructor creates empty payload", () => {
  const payload = new PutPayload();
  expect(payload.contentLength()).toBe(0);
  expect(payload.isEmpty()).toBe(true);
  expect(payload.concat().length).toBe(0);
  expect(payload.chunks().length).toBe(0);
});

test("PutPayload - fromBytes creates payload from Buffer", () => {
  const data = Buffer.from("hello world");
  const payload = PutPayload.fromBytes(data);
  expect(payload.contentLength()).toBe(11);
  expect(payload.isEmpty()).toBe(false);
  expect(payload.concat().toString()).toBe("hello world");

  const chunks = payload.chunks();
  expect(chunks.length).toBeGreaterThan(0);
  expect(Buffer.concat(chunks).toString()).toBe("hello world");

  const cloned = payload.clone();
  expect(cloned.contentLength()).toBe(11);
  expect(cloned.concat().toString()).toBe("hello world");
});

test("PutPayload - fromString creates payload from UTF-8 string", () => {
  const payload = PutPayload.fromString("test UTF-8 🚀");
  expect(payload.contentLength()).toBe(Buffer.byteLength("test UTF-8 🚀"));
  expect(payload.isEmpty()).toBe(false);
  expect(payload.concat().toString()).toBe("test UTF-8 🚀");
});

test("PutPayload - put to ObjectStore and InMemory", async () => {
  const store = ObjectStore.createInMemory();
  const payload = PutPayload.fromString("payload-content");
  await store.put("file.txt", payload);

  const readData = await store.get("file.txt");
  expect(readData.toString()).toBe("payload-content");

  const memStore = new InMemory();
  const payload2 = PutPayload.fromBytes(Buffer.from("mem-content"));
  await memStore.put("mem.txt", payload2);
  const memData = await memStore.get("mem.txt");
  expect(memData.toString()).toBe("mem-content");
});

test("PutPayloadMut - new and withBlockSize create builder", () => {
  const builder = new PutPayloadMut();
  expect(builder.contentLength()).toBe(0);
  expect(builder.isEmpty()).toBe(true);

  const customBuilder = PutPayloadMut.withBlockSize(1024);
  expect(customBuilder.contentLength()).toBe(0);
  expect(customBuilder.isEmpty()).toBe(true);
});

test("PutPayloadMut - push and extendFromSlice append data and freeze into PutPayload", () => {
  const builder = PutPayloadMut.withBlockSize(64);
  builder.push(Buffer.from("chunk1-"));
  builder.extendFromSlice(Buffer.from("slice1-"));
  builder.push(Buffer.from("chunk2"));

  expect(builder.contentLength()).toBe(Buffer.byteLength("chunk1-slice1-chunk2"));
  expect(builder.isEmpty()).toBe(false);

  const frozen = builder.freeze();
  expect(frozen.contentLength()).toBe(Buffer.byteLength("chunk1-slice1-chunk2"));
  expect(frozen.concat().toString()).toBe("chunk1-slice1-chunk2");

  // Builder becomes empty after freeze
  expect(builder.contentLength()).toBe(0);
  expect(builder.isEmpty()).toBe(true);
});

test("PutPayloadMut - put unfrozen PutPayloadMut directly to store", async () => {
  const store = ObjectStore.createInMemory();
  const builder = new PutPayloadMut();
  builder.extendFromSlice(Buffer.from("mutable-put-data"));

  await store.put("mut.txt", builder);
  const data = await store.get("mut.txt");
  expect(data.toString()).toBe("mutable-put-data");
});
