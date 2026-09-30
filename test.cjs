const assert = require("node:assert/strict");
const test = require("node:test");

void test("object store in memory via CJS import", async () => {
  const { ObjectStore, parseUrl } = await import("./index.js");

  const store = ObjectStore.inMemory();
  await store.put("hello.txt", Buffer.from("world"));

  const res = await store.get("hello.txt");
  assert.equal(res.bytes.toString(), "world");
  assert.equal(res.meta.size, 5);

  const parsed = parseUrl("memory://path/to/data");
  assert.ok(parsed.store);
  assert.equal(parsed.path, "path/to/data");
});
