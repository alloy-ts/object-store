import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("copy and copyOpts methods", async () => {
  const store = ObjectStore.memory();
  await store.put("src.txt", Buffer.from("source data"));

  await store.copy("src.txt", "dst.txt");
  const dst = await store.get("dst.txt");
  assert.equal(dst.toString("utf8"), "source data");

  await store.copyOpts("src.txt", "dst2.txt", { mode: "overwrite" });
  const dst2 = await store.get("dst2.txt");
  assert.equal(dst2.toString("utf8"), "source data");
});
