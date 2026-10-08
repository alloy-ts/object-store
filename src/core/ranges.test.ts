import { expect, test } from "vite-plus/test";
import { ObjectStore } from "../../dist/index.js";

test("Ranges - getRanges", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("range_file.txt", Buffer.from("0123456789"));

  const ranges = await store.getRanges("range_file.txt", [
    { start: 0, end: 4 },
    { start: 5, end: 9 },
  ]);

  expect(ranges.length).toBe(2);
  expect(ranges[0]!.toString()).toBe("0123");
  expect(ranges[1]!.toString()).toBe("5678");
});
