import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../index.js";

test("List - list, listOpts and listWithDelimiter", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/b.txt", Buffer.from("b"));

  const list = await store.list("dir");
  expect(list.length).toBe(2);

  const listOpts = await store.listOpts("dir");
  expect(listOpts.length).toBe(2);

  const listDelim = await store.listWithDelimiter("dir/");
  expect(listDelim.objects.length).toBe(2);
});

test("List - listPaginated walks every page with maxKeys", async () => {
  const store = ObjectStore.createInMemory();
  for (const name of ["a", "b", "c", "d", "e"]) {
    await store.put(`dir/${name}.txt`, Buffer.from(name));
  }

  const seen: string[] = [];
  let pageToken: string | null = null;
  let pages = 0;
  do {
    const page = await store.listPaginated("dir", { maxKeys: 2, pageToken });
    pages += 1;
    expect(page.result.objects.length).toBeLessThanOrEqual(2);
    seen.push(...page.result.objects.map((o) => o.location));
    pageToken = page.pageToken;
  } while (pageToken != null);

  expect(pages).toBe(3);
  // Every object exactly once, in listing order, and no duplicates across pages.
  expect(seen).toEqual([
    "dir/a.txt",
    "dir/b.txt",
    "dir/c.txt",
    "dir/d.txt",
    "dir/e.txt",
  ]);
});

test("List - listPaginated without maxKeys returns everything and no token", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/b.txt", Buffer.from("b"));

  const page = await store.listPaginated("dir");
  expect(page.result.objects.length).toBe(2);
  expect(page.pageToken).toBe(null);
});

test("List - listPaginated honours offset and delimiter", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/nested/b.txt", Buffer.from("b"));
  await store.put("dir/nested/c.txt", Buffer.from("c"));
  await store.put("other/d.txt", Buffer.from("d"));

  // The object at `offset` is not included.
  const offsetPage = await store.listPaginated("dir", { offset: "dir/a.txt" });
  expect(offsetPage.result.objects.map((o) => o.location)).toEqual([
    "dir/nested/b.txt",
    "dir/nested/c.txt",
  ]);

  const delimited = await store.listPaginated("dir", { delimiter: "/" });
  expect(delimited.result.objects.map((o) => o.location)).toEqual(["dir/a.txt"]);
  expect(delimited.result.commonPrefixes).toEqual(["dir/nested/"]);
  expect(delimited.pageToken).toBe(null);
});

test("List - listPaginated pages over a delimited listing", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("dir/a.txt", Buffer.from("a"));
  await store.put("dir/b.txt", Buffer.from("b"));
  for (const dir of ["one", "two", "three"]) {
    await store.put(`dir/${dir}/x.txt`, Buffer.from(dir));
  }

  const seen: string[] = [];
  let pageToken: string | null = null;
  do {
    const page = await store.listPaginated("dir", {
      delimiter: "/",
      maxKeys: 2,
      pageToken,
    });
    seen.push(...page.result.objects.map((o) => o.location));
    seen.push(...page.result.commonPrefixes);
    pageToken = page.pageToken;
  } while (pageToken != null);

  expect(seen).toEqual([
    "dir/a.txt",
    "dir/b.txt",
    "dir/one/",
    "dir/three/",
    "dir/two/",
  ]);
});

test("List - listPaginated rejects a page token it did not mint", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("dir/a.txt", Buffer.from("a"));

  await expect(
    store.listPaginated("dir", { pageToken: "dir/a.txt" }),
  ).rejects.toThrow(/malformed page token/);
});
