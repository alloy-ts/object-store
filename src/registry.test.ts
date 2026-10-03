import { expect, test } from "vite-plus/test";
import { ObjectStore, ObjectStoreRegistry } from "../index.js";

test("registry - register/resolve returns longest prefix and trailing path", async () => {
  const registry = new ObjectStoreRegistry();

  const bucket = ObjectStore.createInMemory();
  registry.register("memory:///bucket1/", bucket);

  const [store, path] = registry.resolve("memory:///bucket1/path/to/object");
  expect(path).toBe("path/to/object");

  // The resolved store must be the registered one and usable.
  await store.put("path/to/object", Buffer.from("hello"));
  const got = await store.get("path/to/object");
  expect(got.toString()).toBe("hello");
});

test("registry - register replaces and returns the previous store", () => {
  const registry = new ObjectStoreRegistry();

  const first = ObjectStore.createInMemory();
  const second = ObjectStore.createInMemory();

  expect(registry.register("memory:///x/", first)).toBeNull();
  const previous = registry.register("memory:///x/", second);
  expect(previous).not.toBeNull();
});

test("registry - nested prefixes resolve to the longest match", () => {
  const registry = new ObjectStoreRegistry();

  const outer = ObjectStore.createInMemory();
  const inner = ObjectStore.createInMemory();

  registry.register("mock://bucket/a/", outer);
  registry.register("mock://bucket/a/b/", inner);

  const [store, path] = registry.resolve("mock://bucket/a/b/x");
  expect(path).toBe("x");
});

test("registry - deregister removes exactly the registered url", () => {
  const registry = new ObjectStoreRegistry();

  const store = ObjectStore.createInMemory();
  registry.register("mock://bucket/a/", store);

  // Removing a sibling path under the same authority leaves it in place.
  expect(registry.deregister("mock://bucket/a/b/")).toBeNull();

  const removed = registry.deregister("mock://bucket/a/");
  expect(removed).not.toBeNull();

  // A never-registered URL returns null.
  expect(registry.deregister("mock://other/x/")).toBeNull();

  // mock:// is not understood by parse_url_opts, so a deregistered URL errors.
  expect(() => registry.resolve("mock://bucket/a/foo")).toThrow();
});
