import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("copy method", async () => {
  const store = ObjectStore.memory();
  await store.put("src.txt", Buffer.from("source data"));

  await store.copy("src.txt", "dst.txt");
  const dst = await store.get("dst.txt");
  assert.equal(dst.toString("utf8"), "source data");
});
