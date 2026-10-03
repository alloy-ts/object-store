import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core delete", () => {
  test("delete object", async () => {
    const store = ObjectStore.inMemory();
    await store.put("to_delete.txt", Buffer.from("delete me"));

    const before = await store.list(null);
    expect(before.length).toBe(1);

    await store.delete("to_delete.txt");

    const after = await store.list(null);
    expect(after.length).toBe(0);
  });
});
