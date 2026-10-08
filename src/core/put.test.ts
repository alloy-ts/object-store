import { expect, test } from "vite-plus/test";
import { ObjectStore, InMemory, MultipartStore, PutPayload, PutPayloadMut } from "../../dist/index.js";

const toStr = (b: Buffer) => b.toString();

test("Put - put and putOpts basic writes", async () => {
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

  const res3 = await store.putOpts("file.txt", Buffer.from("opts content"), {
    modeOverwrite: true,
  });
  expect(res3).toBeDefined();

  const data3 = await store.get("file.txt");
  expect(data3.toString()).toBe("opts content");
});

test("Put - modeCreate succeeds on new file and fails on existing file", async () => {
  const store = ObjectStore.createInMemory();

  // Mode create on a new file should succeed
  await store.put("created.txt", Buffer.from("initial"), {
    modeCreate: true,
  });
  expect((await store.get("created.txt")).toString()).toBe("initial");

  // Mode create on an existing file should reject
  await expect(
    store.put("created.txt", Buffer.from("second"), {
      modeCreate: true,
    })
  ).rejects.toThrow();
});

test("Put - empty payload write", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("empty.txt", Buffer.alloc(0));

  const data = await store.get("empty.txt");
  expect(data.length).toBe(0);

  const meta = await store.getWithMeta("empty.txt");
  expect(meta.meta.size).toBe(0);
});

test("Put - tags and attributes options", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("attr.txt", Buffer.from("data with attributes"), {
    tags: { env: "test", project: "alloy" },
    attributes: {
      "content-type": "text/plain",
      "cache-control": "no-cache",
      custom_meta: "custom_value",
    },
  });

  const res = await store.getWithMeta("attr.txt");
  expect(res.bytes.toString()).toBe("data with attributes");
  expect(res.attributes["content-type"]).toBe("text/plain");
  expect(res.attributes["cache-control"]).toBe("no-cache");
  expect(res.attributes["custom_meta"]).toBe("custom_value");
});

test("Put - empty attribute key rejected", async () => {
  const store = ObjectStore.createInMemory();
  await expect(
    store.put("invalid.txt", Buffer.from("data"), {
      attributes: { "": "invalid" },
    })
  ).rejects.toThrow(/attribute keys must not be empty/);
});

test("PutMultipart - complete workflow assembling multiple parts", async () => {
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipart("multipart.txt");
  await upload.putPart(Buffer.from("part 1 - "));

  const payloadPart = PutPayload.fromString("part 2 - ");
  await upload.putPart(payloadPart);

  const mutPart = new PutPayloadMut();
  mutPart.extendFromSlice(Buffer.from("part 3"));
  await upload.putPart(mutPart.freeze());

  const result = await upload.complete();
  expect(result).toBeDefined();

  const finalData = await store.get("multipart.txt");
  expect(finalData.toString()).toBe("part 1 - part 2 - part 3");
});

test("PutMultipart - putMultipartOpts with tags and attributes", async () => {
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipartOpts("mp_opts.txt", {
    tags: { tag1: "val1" },
    attributes: { "content-type": "application/json" },
  });
  await upload.putPart(Buffer.from('{"hello":"world"}'));
  await upload.complete();

  const res = await store.getWithMeta("mp_opts.txt");
  expect(res.bytes.toString()).toBe('{"hello":"world"}');
  expect(res.attributes["content-type"]).toBe("application/json");
});

test("PutMultipart - abort workflow discards upload", async () => {
  const store = ObjectStore.createInMemory();

  const upload = await store.putMultipart("aborted.txt");
  await upload.putPart(Buffer.from("some data"));
  await upload.abort();

  await expect(store.get("aborted.txt")).rejects.toThrow();
});

test("PutPayload - constructor and static methods", () => {
  const payload = new PutPayload();
  expect(payload.contentLength()).toBe(0);
  expect(payload.isEmpty()).toBe(true);
  expect(payload.concat().length).toBe(0);
  expect(payload.chunks().length).toBe(0);

  const bytesPayload = PutPayload.fromBytes(Buffer.from("hello world"));
  expect(bytesPayload.contentLength()).toBe(11);
  expect(bytesPayload.isEmpty()).toBe(false);
  expect(bytesPayload.concat().toString()).toBe("hello world");

  const chunks = bytesPayload.chunks();
  expect(chunks.length).toBeGreaterThan(0);
  expect(Buffer.concat(chunks).toString()).toBe("hello world");

  const cloned = bytesPayload.clone();
  expect(cloned.contentLength()).toBe(11);
  expect(cloned.concat().toString()).toBe("hello world");

  const stringPayload = PutPayload.fromString("test UTF-8 🚀");
  expect(stringPayload.contentLength()).toBe(Buffer.byteLength("test UTF-8 🚀"));
  expect(stringPayload.isEmpty()).toBe(false);
  expect(stringPayload.concat().toString()).toBe("test UTF-8 🚀");
});

test("PutPayload - put to ObjectStore and InMemory", async () => {
  const store = ObjectStore.createInMemory();
  const payload = PutPayload.fromString("payload-content");
  await store.put("file.txt", payload);

  const readData = await store.get("file.txt");
  expect(readData.toString()).toBe("payload-content");

  const memStore = new InMemory();
  const payload2 = PutPayload.fromBytes(Buffer.from("mem-content"));
  await memStore.asObjectStore().put("mem.txt", payload2);
  const memData = await memStore.asObjectStore().get("mem.txt");
  expect(memData.toString()).toBe("mem-content");
});

test("PutPayloadMut - builder and direct put", async () => {
  const builder = new PutPayloadMut();
  expect(builder.contentLength()).toBe(0);
  expect(builder.isEmpty()).toBe(true);

  const customBuilder = PutPayloadMut.withBlockSize(64);
  customBuilder.push(Buffer.from("chunk1-"));
  customBuilder.extendFromSlice(Buffer.from("slice1-"));
  customBuilder.push(Buffer.from("chunk2"));

  expect(customBuilder.contentLength()).toBe(Buffer.byteLength("chunk1-slice1-chunk2"));
  expect(customBuilder.isEmpty()).toBe(false);

  const frozen = customBuilder.freeze();
  expect(frozen.contentLength()).toBe(Buffer.byteLength("chunk1-slice1-chunk2"));
  expect(frozen.concat().toString()).toBe("chunk1-slice1-chunk2");

  // Builder becomes empty after freeze
  expect(customBuilder.contentLength()).toBe(0);
  expect(customBuilder.isEmpty()).toBe(true);

  const store = ObjectStore.createInMemory();
  const directMut = new PutPayloadMut();
  directMut.extendFromSlice(Buffer.from("mutable-put-data"));

  await store.put("mut.txt", directMut.freeze());
  const data = await store.get("mut.txt");
  expect(data.toString()).toBe("mutable-put-data");
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

test("PutPayload - usable as a multipart part via MultipartStore", async () => {
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
