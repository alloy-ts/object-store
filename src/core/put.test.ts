import { expect, test } from "vite-plus/test";
import {
  InMemory,
  MultipartStore,
  ObjectStore,
  PutPayload,
  PutPayloadMut,
} from "../../dist/index.js";

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

test("Put - put forwards tags and attributes that read back on the object", async () => {
  const store = ObjectStore.createInMemory();

  await store.put("attributed.bin", Buffer.from("payload"), {
    tags: { team: "storage" },
    attributes: {
      "content-type": "application/octet-stream",
      "cache-control": "max-age=60",
      "custom-key": "custom-value",
    },
  });

  const res = await store.getWithMeta("attributed.bin");
  expect(res.attributes["content-type"]).toBe("application/octet-stream");
  expect(res.attributes["cache-control"]).toBe("max-age=60");
  // Unrecognised keys round-trip as user-defined metadata.
  expect(res.attributes["custom-key"]).toBe("custom-value");
});

test("Put - put rejects an empty attribute key", async () => {
  const store = ObjectStore.createInMemory();

  await expect(
    store.put("bad.bin", Buffer.from("payload"), { attributes: { "": "value" } }),
  ).rejects.toBeDefined();
});

test("Put - putMultipart forwards attributes to the completed object", async () => {
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipartOpts("tagged.bin", {
    tags: { team: "storage" },
    attributes: { "content-type": "text/plain" },
  });
  await upload.putPart(Buffer.from("chunk"));
  await upload.complete();

  const res = await store.getWithMeta("tagged.bin");
  expect(res.attributes["content-type"]).toBe("text/plain");
  expect((await store.get("tagged.bin")).toString()).toBe("chunk");
});

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
