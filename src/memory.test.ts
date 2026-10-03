import { expect, test } from "vite-plus/test";
import { InMemory } from "../index.js";

test("InMemory - new, put, get round-trip", async () => {
  const store = new InMemory();
  await store.put("a/b.txt", Buffer.from("hello"));
  const data = await store.get("a/b.txt");
  expect(data.toString()).toBe("hello");
});

test("InMemory - fork snapshots content and diverges", async () => {
  const store = new InMemory();
  await store.put("shared.txt", Buffer.from("v1"));

  const forked = store.fork();

  // Mutate the original; the fork must keep its snapshot.
  await store.put("shared.txt", Buffer.from("v2"));
  await forked.put("fork-only.txt", Buffer.from("forked"));

  expect((await store.get("shared.txt")).toString()).toBe("v2");
  expect((await forked.get("shared.txt")).toString()).toBe("v1");
  expect((await forked.get("fork-only.txt")).toString()).toBe("forked");

  // Original does not see the fork-only key.
  const originalList = await store.list();
  expect(originalList.map((m) => m.location)).not.toContain("fork-only.txt");
});
