const assert = require("node:assert/strict");
const test = require("node:test");
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");

void test("ObjectStore - in memory put, get, head, delete, list, copy, rename, getRanges", async () => {
  const { ObjectStore } = await import("./index.js");
  const store = ObjectStore.createInMemory();

  // put & get
  await store.put("file1.txt", Buffer.from("hello world"));
  const data = await store.get("file1.txt");
  assert.equal(data.toString(), "hello world");

  // getWithMeta
  const getWithMetaRes = await store.getWithMeta("file1.txt");
  assert.equal(getWithMetaRes.bytes.toString(), "hello world");
  assert.equal(getWithMetaRes.meta.location, "file1.txt");
  assert.equal(getWithMetaRes.meta.size, 11);

  // head
  const meta = await store.head("file1.txt");
  assert.equal(meta.location, "file1.txt");
  assert.equal(meta.size, 11);

  // copy
  await store.copy("file1.txt", "file2.txt");
  const dataCopy = await store.get("file2.txt");
  assert.equal(dataCopy.toString(), "hello world");

  // rename
  await store.rename("file2.txt", "file3.txt");
  const dataRename = await store.get("file3.txt");
  assert.equal(dataRename.toString(), "hello world");
  await assert.rejects(async () => store.get("file2.txt"));

  // getRanges
  const ranges = await store.getRanges("file1.txt", [
    { start: 0, end: 5 },
    { start: 6, end: 11 },
  ]);
  assert.equal(ranges.length, 2);
  assert.equal(ranges[0].toString(), "hello");
  assert.equal(ranges[1].toString(), "world");

  // list
  const listItems = await store.list();
  const locations = listItems.map((i) => i.location).sort();
  assert.deepEqual(locations, ["file1.txt", "file3.txt"]);

  // listWithDelimiter
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/b.txt", Buffer.from("b"));
  const delimRes = await store.listWithDelimiter("dir/");
  assert.equal(delimRes.objects.length, 2);

  // delete
  await store.delete("file1.txt");
  await assert.rejects(async () => store.get("file1.txt"));
});

void test("ObjectStore - local file system store", async () => {
  const { ObjectStore } = await import("./index.js");
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "object-store-test-"));

  try {
    const store = ObjectStore.createLocal(tmpDir);
    await store.put("local_test.txt", Buffer.from("local content"));
    const data = await store.get("local_test.txt");
    assert.equal(data.toString(), "local content");

    const meta = await store.head("local_test.txt");
    assert.equal(meta.size, 13);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});
