import { describe, expect, test } from "vite-plus/test";
import { ObjectStore } from "../main.ts";

describe("core get", () => {
  test("get, getRange, getRanges, getOpts", async () => {
    const store = ObjectStore.inMemory();
    await store.put("data.txt", Buffer.from("0123456789abcdefghij"));

    const getRes = await store.get("data.txt");
    expect(getRes.bytes.toString()).toBe("0123456789abcdefghij");

    const range = await store.getRange("data.txt", 0, 10);
    expect(range.toString()).toBe("0123456789");

    const ranges = await store.getRanges("data.txt", [
      { start: 0, length: 5 },
      { start: 10, length: 5 },
    ]);
    expect(ranges.length).toBe(2);
    expect(ranges[0]?.toString()).toBe("01234");
    expect(ranges[1]?.toString()).toBe("abcde");

    const optsRes = await store.getOpts("data.txt", {
      range: { start: 5, length: 5 },
    });
    expect(optsRes.bytes.toString()).toBe("56789");
  });
});
