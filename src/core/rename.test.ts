import assert from "node:assert/strict";
import test from "node:test";

// @ts-ignore
import { ObjectStore } from "../../index.js";

test("rename method", async () => {
  const store = ObjectStore.memory();
  await store.put("old.txt", Buffer.from("renamed data"));

  await store.rename("old.txt", "new.txt");
  const data = await store.get("new.txt");
  assert.equal(data.toString("utf8"), "renamed data");

  await assert.rejects(async () => {
    await store.get("old.txt");
  });
});
