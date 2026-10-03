import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core copy", () => {
  test("copy object", async () => {
    const store = ObjectStore.inMemory();
    await store.put("source.txt", Buffer.from("copy payload"));

    await store.copy("source.txt", "dest.txt");

    const src = await store.get("source.txt");
    const dst = await store.get("dest.txt");
    expect(src.bytes.toString()).toBe("copy payload");
    expect(dst.bytes.toString()).toBe("copy payload");
  });
});
