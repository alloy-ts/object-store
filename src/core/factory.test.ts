import { describe, expect, test } from "vite-plus/test";
import { ObjectStore, parseUrl } from "../main.ts";

describe("core factory", () => {
  test("inMemory, local, fromUrl, parseUrl", async () => {
    const memStore = ObjectStore.inMemory();
    await memStore.put("a.txt", Buffer.from("a"));
    expect((await memStore.get("a.txt")).bytes.toString()).toBe("a");

    const urlStore = ObjectStore.fromUrl("memory://");
    await urlStore.put("b.txt", Buffer.from("b"));
    expect((await urlStore.get("b.txt")).bytes.toString()).toBe("b");

    const parsed = parseUrl("memory://path/to/resource");
    expect(parsed.store).toBeDefined();
    expect(parsed.path).toBe("path/to/resource");
  });
});
