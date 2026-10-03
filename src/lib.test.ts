import { describe, expect, test } from "vite-plus/test";
import { ObjectStore, parseUrl } from "./main.ts";

describe("object_store core API examples", () => {
  test("List objects", async () => {
    const store = ObjectStore.inMemory();

    await store.put("data/file01.parquet", Buffer.from("data1"));
    await store.put("data/file02.parquet", Buffer.from("data2"));
    await store.put("data/child/file03.parquet", Buffer.from("data3"));

    const list = await store.list("data");
    expect(list.length).toBe(3);

    const locations = list.map((item) => item.location).sort();
    expect(locations).toEqual([
      "data/child/file03.parquet",
      "data/file01.parquet",
      "data/file02.parquet",
    ]);
  });

  test("Fetch objects (head, get, bytes)", async () => {
    const store = ObjectStore.inMemory();
    const path = "data/file01.parquet";
    const payload = Buffer.from("hello world content");

    await store.put(path, payload);

    // Fetch just the file metadata
    const meta = await store.head(path);
    expect(meta.location).toBe(path);
    expect(meta.size).toBe(payload.length);

    // Fetch the object including metadata
    const result = await store.get(path);
    expect(result.meta.location).toBe(meta.location);
    expect(result.meta.size).toBe(meta.size);

    // Buffer the entire object in memory
    expect(result.bytes.length).toBe(meta.size);
    expect(result.bytes.toString()).toBe("hello world content");
  });

  test("Put Object", async () => {
    const store = ObjectStore.inMemory();
    const path = "data/file1";
    const payload = Buffer.from("hello");

    await store.put(path, payload);

    const getRes = await store.get(path);
    expect(getRes.bytes.toString()).toBe("hello");
  });

  test("Vectored Read (getRanges)", async () => {
    const store = ObjectStore.inMemory();
    const path = "data/large_file";

    const buffer = Buffer.alloc(1000);
    buffer.write("1234567890", 90);
    buffer.write("abcdefghij", 400);
    buffer.write("0123456789", 0);

    await store.put(path, buffer);

    const ranges = await store.getRanges(path, [
      { start: 90, length: 10 },
      { start: 400, length: 10 },
      { start: 0, length: 10 },
    ]);

    expect(ranges.length).toBe(3);
    expect(ranges[0]?.toString()).toBe("1234567890");
    expect(ranges[1]?.toString()).toBe("abcdefghij");
    expect(ranges[2]?.toString()).toBe("0123456789");
  });

  test("URL parsing / parseUrl", async () => {
    const parsed = parseUrl("memory://data/path");
    expect(parsed.store).toBeDefined();
    expect(parsed.path).toBe("data/path");

    await parsed.store.put("file.txt", Buffer.from("parsed url test"));
    const fetched = await parsed.store.get("file.txt");
    expect(fetched.bytes.toString()).toBe("parsed url test");
  });
});
