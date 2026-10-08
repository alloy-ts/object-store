import { expect, test } from "vite-plus/test";
import { InMemory } from "../index.js";

test("InMemory - new, put, get round-trip via the core ObjectStore API", async () => {
  const store = new InMemory().asObjectStore();
  await store.put("a/b.txt", Buffer.from("hello"));
  const data = await store.get("a/b.txt");
  expect(data.toString()).toBe("hello");
});

test("InMemory - fork snapshots content and diverges", async () => {
  const store = new InMemory();
  const original = store.asObjectStore();
  await original.put("shared.txt", Buffer.from("v1"));

  const forked = store.fork();
  const forkedView = forked.asObjectStore();

  // Mutate the original; the fork must keep its snapshot.
  await original.put("shared.txt", Buffer.from("v2"));
  await forkedView.put("fork-only.txt", Buffer.from("forked"));

  expect((await original.get("shared.txt")).toString()).toBe("v2");
  expect((await forkedView.get("shared.txt")).toString()).toBe("v1");
  expect((await forkedView.get("fork-only.txt")).toString()).toBe("forked");

  // Original does not see the fork-only key.
  const originalList = await original.list();
  expect(originalList.map((m) => m.location)).not.toContain("fork-only.txt");
});

test("InMemory - asObjectStore shares the backing storage", async () => {
  const store = new InMemory();
  const view = store.asObjectStore();
  await view.put("x.txt", Buffer.from("1"));

  // A second view of the same store observes the same data.
  expect((await store.asObjectStore().get("x.txt")).toString()).toBe("1");
});
