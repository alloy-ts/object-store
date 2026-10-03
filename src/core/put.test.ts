import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core put", () => {
  test("put and putOpts", async () => {
    const store = ObjectStore.inMemory();

    const putRes = await store.put("file.txt", Buffer.from("hello"));
    expect(putRes).toBeDefined();

    const getRes = await store.get("file.txt");
    expect(getRes.bytes.toString()).toBe("hello");

    const putOptsRes = await store.putOpts("file2.txt", Buffer.from("world"), {
      mode: "create",
    });
    expect(putOptsRes).toBeDefined();

    const getRes2 = await store.get("file2.txt");
    expect(getRes2.bytes.toString()).toBe("world");
  });
});
