import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core list", () => {
  test("list and listWithDelimiter", async () => {
    const store = ObjectStore.inMemory();
    await store.put("data/file1.txt", Buffer.from("1"));
    await store.put("data/file2.txt", Buffer.from("2"));
    await store.put("data/sub/file3.txt", Buffer.from("3"));

    const listAll = await store.list("data");
    expect(listAll.length).toBe(3);

    const listDelim = await store.listWithDelimiter("data");
    expect(listDelim.objects.length).toBe(2);
    expect(listDelim.commonPrefixes).toContain("data/sub");
  });
});
