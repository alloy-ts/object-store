import { expect, test } from "vite-plus/test";
import {
  InMemory,
  MultipartStore,
  ObjectStore,
  PutPayload,
  PutPayloadMut,
} from "../index.js";

const toStr = (b: Buffer) => b.toString();

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

// --- PutPayload assembly & inspection ----------------------------------------

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
  // Block size 5 with 12 bytes spans three blocks; the reassembled bytes must
  // be identical to the input regardless of block boundaries.
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

// --- PutPayloadMut frozen into a PutPayload, then used by put ----------------

test("PutPayloadMut - freeze then put round-trips", async () => {
  const store = ObjectStore.createInMemory();
  const builder = new PutPayloadMut();
  builder.push(Buffer.from("frozen-then-put"));
  const payload = builder.freeze();
  await store.put("m.bin", payload);
  expect(toStr(await store.get("m.bin"))).toBe("frozen-then-put");
});

// --- PutPayload as a multipart part ------------------------------------------

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

// --- error path --------------------------------------------------------------

test("PutPayload - non-payload/non-buffer value is rejected by put", () => {
  const store = ObjectStore.createInMemory();
  // NAPI validates arguments synchronously, before the async call is issued.
  // @ts-expect-error - 12345 is neither a Buffer nor a PutPayload
  expect(() => store.put("x.bin", 12345)).toThrow(/none of these types/);
});
