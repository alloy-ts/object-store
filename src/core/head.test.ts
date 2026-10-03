import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core head", () => {
  test("head metadata retrieval", async () => {
    const store = ObjectStore.inMemory();
    const content = Buffer.from("metadata test content");
    await store.put("meta.txt", content);

    const meta = await store.head("meta.txt");
    expect(meta.location).toBe("meta.txt");
    expect(meta.size).toBe(content.length);
    expect(meta.lastModified).toBeDefined();
  });
});
