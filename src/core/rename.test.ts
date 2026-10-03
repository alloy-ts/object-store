import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core rename", () => {
  test("rename object", async () => {
    const store = ObjectStore.inMemory();
    await store.put("old_name.txt", Buffer.from("rename payload"));

    await store.rename("old_name.txt", "new_name.txt");

    const renamed = await store.get("new_name.txt");
    expect(renamed.bytes.toString()).toBe("rename payload");

    const list = await store.list(null);
    expect(list.length).toBe(1);
    expect(list[0]?.location).toBe("new_name.txt");
  });
});
