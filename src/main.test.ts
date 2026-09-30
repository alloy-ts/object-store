import { describe, expect, test } from "vite-plus/test";
import { ObjectStore, parseUrl } from "./main.ts";

describe("ObjectStore inMemory", () => {
  test("put, get, head, getRange, list, copy, rename, delete", async () => {
    const store = ObjectStore.inMemory();

    // put
    const content = Buffer.from("Hello Object Store!");
    await store.put("file1.txt", content);

    // get
    const getRes = await store.get("file1.txt");
    expect(getRes.bytes.toString()).toBe("Hello Object Store!");
    expect(getRes.meta.size).toBe(content.length);
    expect(getRes.meta.location).toBe("file1.txt");

    // head
    const headRes = await store.head("file1.txt");
    expect(headRes.size).toBe(content.length);
    expect(headRes.location).toBe("file1.txt");

    // getRange
    const rangeBytes = await store.getRange("file1.txt", 0, 5);
    expect(rangeBytes.toString()).toBe("Hello");

    // list
    const listRes = await store.list(null);
    expect(listRes.length).toBe(1);
    expect(listRes[0]?.location).toBe("file1.txt");

    // copy
    await store.copy("file1.txt", "file2.txt");
    const copyRes = await store.get("file2.txt");
    expect(copyRes.bytes.toString()).toBe("Hello Object Store!");

    // list with delimiter
    const listDelim = await store.listWithDelimiter(null);
    expect(listDelim.objects.length).toBe(2);

    // rename
    await store.rename("file2.txt", "file3.txt");
    const renameRes = await store.get("file3.txt");
    expect(renameRes.bytes.toString()).toBe("Hello Object Store!");

    // delete
    await store.delete("file3.txt");
    const listAfterDelete = await store.list(null);
    expect(listAfterDelete.length).toBe(1);
  });

  test("factory constructors and parseUrl", async () => {
    const storeFromUrl = ObjectStore.fromUrl("memory://");
    await storeFromUrl.put("test.txt", Buffer.from("data"));
    const getRes = await storeFromUrl.get("test.txt");
    expect(getRes.bytes.toString()).toBe("data");

    const parsed = parseUrl("memory://some/path/file.txt");
    expect(parsed.store).toBeDefined();
    expect(parsed.path).toBe("some/path/file.txt");
  });
});
