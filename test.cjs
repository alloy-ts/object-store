const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { ObjectStore, parseUrl } = require("./index.js");

test("memory store - basic put, get, head, delete", async () => {
  const store = ObjectStore.memory();

  const payload = Buffer.from("hello world");
  const putRes = await store.put("data/test1.txt", payload);
  assert.ok(putRes);

  const data = await store.get("data/test1.txt");
  assert.equal(data.toString("utf8"), "hello world");

  const meta = await store.head("data/test1.txt");
  assert.equal(meta.location, "data/test1.txt");
  assert.equal(meta.size, 11);
  assert.ok(meta.lastModified);

  await store.delete("data/test1.txt");
  await assert.rejects(async () => {
    await store.get("data/test1.txt");
  });
});

test("memory store - list, copy, rename, getRanges, getOpts", async () => {
  const store = new ObjectStore();

  await store.put("folder/file1.txt", Buffer.from("0123456789"));
  await store.put("folder/file2.txt", Buffer.from("abcdefghij"));

  const list = await store.list("folder");
  assert.equal(list.length, 2);
  const locations = list.map((item) => item.location).sort();
  assert.deepEqual(locations, ["folder/file1.txt", "folder/file2.txt"]);

  // getOpts with range
  const partial = await store.getOpts("folder/file1.txt", {
    rangeStart: 2,
    rangeEnd: 5,
  });
  assert.equal(partial.toString("utf8"), "234");

  // getRanges
  const ranges = await store.getRanges("folder/file1.txt", [
    { start: 0, end: 3 },
    { start: 5, end: 8 },
  ]);
  assert.equal(ranges.length, 2);
  assert.equal(ranges[0].toString("utf8"), "012");
  assert.equal(ranges[1].toString("utf8"), "567");

  // copy
  await store.copy("folder/file1.txt", "folder/file1_copy.txt");
  const copyData = await store.get("folder/file1_copy.txt");
  assert.equal(copyData.toString("utf8"), "0123456789");

  // rename
  await store.rename("folder/file2.txt", "folder/file2_moved.txt");
  const movedData = await store.get("folder/file2_moved.txt");
  assert.equal(movedData.toString("utf8"), "abcdefghij");
  await assert.rejects(async () => {
    await store.get("folder/file2.txt");
  });
});

test("local store - operations", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "obj-store-test-"));
  try {
    const store = ObjectStore.local(tmpDir);

    await store.put("test.txt", Buffer.from("local store content"));
    const data = await store.get("test.txt");
    assert.equal(data.toString("utf8"), "local store content");

    const list = await store.list();
    assert.equal(list.length, 1);
    assert.equal(list[0].location, "test.txt");

    await store.delete("test.txt");
    const listAfterDelete = await store.list();
    assert.equal(listAfterDelete.length, 0);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("parseUrl helper and factory method", async () => {
  const res = parseUrl("memory://");
  assert.ok(res.store);

  await res.store.put("parsed.txt", Buffer.from("parsed content"));
  const fetched = await res.store.get("parsed.txt");
  assert.equal(fetched.toString("utf8"), "parsed content");

  const res2 = ObjectStore.parseUrl("memory://");
  assert.ok(res2.store);
});
