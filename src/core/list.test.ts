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
