import { expect, test } from "vite-plus/test";
import { ObjectStore, InMemory, PutPayload, PutPayloadMut, TagSet } from "../../dist/index.js";

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

test("Put - put with options modeCreate and modeUpdate", async () => {
  const store = ObjectStore.createInMemory();

  // modeCreate on a new file should succeed
  const resCreate = await store.put("created.txt", Buffer.from("initial"), {
    modeCreate: true,
  });
  expect(resCreate).toBeDefined();

  // modeCreate on existing file should fail (AlreadyExists)
  await expect(
    store.put("created.txt", Buffer.from("overwrite"), {
      modeCreate: true,
    })
  ).rejects.toThrow();

  // modeUpdate with matching version
  if (resCreate.eTag || resCreate.version) {
    const resUpdate = await store.put("created.txt", Buffer.from("updated"), {
      modeUpdate: { eTag: resCreate.eTag, version: resCreate.version },
    });
    expect(resUpdate).toBeDefined();
    expect((await store.get("created.txt")).toString()).toBe("updated");
  }
});

test("Put - put with tags and attributes", async () => {
  const store = ObjectStore.createInMemory();

  await store.put("tagged.txt", Buffer.from("tagged content"), {
    tags: { env: "test", project: "alloy" },
    attributes: { "content-type": "text/plain", "custom-attr": "val" },
  });

  const getRes = await store.getWithMeta("tagged.txt");
  expect(getRes.attributes["content-type"]).toBe("text/plain");
  expect(getRes.attributes["custom-attr"]).toBe("val");
});

test("Put - vectored / gathered write via Buffer[]", async () => {
  const store = ObjectStore.createInMemory();
  const buffers = [
    Buffer.from("part1-"),
    Buffer.from("part2-"),
    Buffer.from("part3"),
  ];

  await store.put("vectored.txt", buffers);
  const data = await store.get("vectored.txt");
  expect(data.toString()).toBe("part1-part2-part3");
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
  await memStore.asObjectStore().put("mem.txt", payload2);
  const memData = await memStore.asObjectStore().get("mem.txt");
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

test("MultipartUpload - putMultipart and putPart flow", async () => {
  const store = ObjectStore.createInMemory();
  const upload = await store.putMultipart("large.txt");

  await upload.putPart(Buffer.from("chunk 1 "));
  await upload.putPart(PutPayload.fromString("chunk 2 "));
  await upload.putPart([Buffer.from("chunk 3")]);

  const res = await upload.complete();
  expect(res).toBeDefined();

  const fullData = await store.get("large.txt");
  expect(fullData.toString()).toBe("chunk 1 chunk 2 chunk 3");
});

test("MultipartUpload - abort discards upload", async () => {
  const store = ObjectStore.createInMemory();
  const upload = await store.putMultipart("aborted.txt");

  await upload.putPart(Buffer.from("partial data"));
  await upload.abort();

  await expect(store.get("aborted.txt")).rejects.toThrow();
});

test("TagSet - push, encoded and isEmpty", () => {
  const tags = new TagSet();
  expect(tags.isEmpty()).toBe(true);

  tags.push("test/foo", "value sdlks");
  tags.push("foo", " sdf _ /+./sd");

  expect(tags.isEmpty()).toBe(false);
  expect(tags.encoded()).toBe("test%2Ffoo=value+sdlks&foo=+sdf+_+%2F%2B.%2Fsd");
});
