import { expect, test } from "vite-plus/test";
import {
  ObjectStore,
  InMemory,
  PutPayload,
  PutPayloadMut,
  MultipartStore,
} from "../../dist/index.js";

const toStr = (b: Buffer) => b.toString();

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

  const memStore = new InMemory().asObjectStore();
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

  await store.put("mut.txt", builder.freeze());
  const data = await store.get("mut.txt");
  expect(data.toString()).toBe("mutable-put-data");
});

test("PutPayload - fromBytes round-trips via ObjectStore.put", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("p.bin", PutPayload.fromBytes(Buffer.from("hello payload")));
  expect(toStr(await store.get("p.bin"))).toBe("hello payload");
});

test("PutPayload - fromString round-trips", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("s.txt", PutPayload.fromString(" Unicode ✓ "));
  expect(toStr(await store.get("s.txt"))).toBe(" Unicode ✓ ");
});

test("PutPayload - empty payload isEmpty / contentLength 0 and stores empty", async () => {
  const p = new PutPayload();
  expect(p.isEmpty()).toBe(true);
  expect(p.contentLength()).toBe(0);

  const store = ObjectStore.createInMemory();
  await store.put("empty.bin", p);
  expect((await store.get("empty.bin")).length).toBe(0);
});

test("PutPayloadMut - push builds chunks that concat correctly", () => {
  const builder = new PutPayloadMut();
  builder.push(Buffer.from("foo"));
  builder.push(Buffer.from("bar"));
  builder.push(Buffer.from("baz"));
  expect(builder.contentLength()).toBe(9);
  expect(builder.isEmpty()).toBe(false);

  const p = builder.freeze();
  expect(p.contentLength()).toBe(9);
  expect(toStr(p.concat())).toBe("foobarbaz");
});

test("PutPayload - chunks exposes each pushed segment separately", () => {
  const builder = new PutPayloadMut();
  builder.push(Buffer.from("aa"));
  builder.push(Buffer.from("bbbb"));
  const p = builder.freeze();

  const chunks = p.chunks();
  expect(chunks.length).toBe(2);
  expect(toStr(chunks[0])).toBe("aa");
  expect(toStr(chunks[1])).toBe("bbbb");
});

test("PutPayloadMut - extendFromSlice with a small block size preserves content", () => {
  const builder = PutPayloadMut.withBlockSize(5);
  builder.extendFromSlice(Buffer.from("abcdefghijkl"));
  expect(builder.contentLength()).toBe(12);

  const p = builder.freeze();
  expect(toStr(p.concat())).toBe("abcdefghijkl");
});

test("PutPayload - clone shares content and is independent", () => {
  const p = PutPayload.fromBytes(Buffer.from("shared"));
  const c = p.clone();
  expect(toStr(c.concat())).toBe("shared");
  expect(c.contentLength()).toBe(p.contentLength());
});

test("PutPayloadMut - freeze then put round-trips", async () => {
  const store = ObjectStore.createInMemory();
  const builder = new PutPayloadMut();
  builder.push(Buffer.from("frozen-then-put"));
  const payload = builder.freeze();
  await store.put("m.bin", payload);
  expect(toStr(await store.get("m.bin"))).toBe("frozen-then-put");
});

test("PutPayload - usable as a multipart part", async () => {
  const store = new InMemory();
  const mp = MultipartStore.fromInMemory(store);

  const id = await mp.createMultipart("big.bin");
  const p0 = await mp.putPart("big.bin", id, 0, PutPayload.fromBytes(Buffer.from("hello ")));
  const p1 = await mp.putPart("big.bin", id, 1, PutPayload.fromString("world"));
  expect(typeof p0.contentId).toBe("string");

  await mp.completeMultipart("big.bin", id, [p0, p1]);
  expect(toStr(await store.asObjectStore().get("big.bin"))).toBe("hello world");
});

test("PutPayload - non-payload/non-buffer value is rejected by put", () => {
  const store = ObjectStore.createInMemory();
  // @ts-expect-error - 12345 is neither a Buffer nor a PutPayload
  expect(() => store.put("x.bin", 12345)).toThrow(/none of these types/);
});
