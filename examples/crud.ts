import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ObjectStore, getFeatures, parseUrl } from "../index.js";

async function main() {
  console.log("=== ObjectStore Core API CRUD Example ===");

  console.log("\nActive Feature Flags:", getFeatures());

  // 1. Create in-memory store
  const store = ObjectStore.memory();

  // 2. Put object with options and tags (Create)
  console.log("\n1. PUT object with options & tags");
  const putResult = await store.putOpts("docs/hello.txt", Buffer.from("Hello Object Store!"), {
    mode: "overwrite",
    tags: { category: "documentation", author: "jules" },
  });
  console.log("Put result:", putResult);

  // 3. Head object (Read Metadata)
  console.log("\n2. HEAD object");
  const meta = await store.head("docs/hello.txt");
  console.log("Object metadata:", meta);
  assert.equal(meta.location, "docs/hello.txt");
  assert.equal(meta.size, 19);

  // 4. Get object and GetResult (Read Content)
  console.log("\n3. GET object & getResult");
  const content = await store.get("docs/hello.txt");
  console.log("Fetched content:", content.toString("utf8"));
  assert.equal(content.toString("utf8"), "Hello Object Store!");

  const fullResult = await store.getResult("docs/hello.txt");
  console.log("Full GetResult meta:", fullResult.meta.location, "size:", fullResult.meta.size);
  assert.equal(fullResult.bytes.toString("utf8"), "Hello Object Store!");

  // 5. GetOpts object (Partial Range / Range read & Conditional)
  console.log("\n4. GET with options (Range & Conditional)");
  const partial = await store.getOpts("docs/hello.txt", {
    range: { start: 0, end: 5 },
    ifModifiedSince: new Date(0).toISOString(),
    head: false,
  });
  console.log("Range GET (0..5):", partial.toString("utf8"));
  assert.equal(partial.toString("utf8"), "Hello");

  // 6. GetRanges (Vectored Read)
  console.log("\n5. GET ranges (Vectored Read)");
  const ranges = await store.getRanges("docs/hello.txt", [
    { start: 0, end: 5 },
    { start: 6, end: 12 },
  ]);
  console.log("Range 1 (0..5):", ranges[0]!.toString("utf8"));
  console.log("Range 2 (6..12):", ranges[1]!.toString("utf8"));

  // 7. Copy & CopyOpts object
  console.log("\n6. COPY & copyOpts object");
  await store.copy("docs/hello.txt", "docs/hello_copy.txt");
  await store.copyOpts("docs/hello.txt", "docs/hello_copy2.txt", { mode: "overwrite" });
  const copiedContent = await store.get("docs/hello_copy.txt");
  assert.equal(copiedContent.toString("utf8"), "Hello Object Store!");

  // 8. Rename & RenameOpts object
  console.log("\n7. RENAME & renameOpts object");
  await store.rename("docs/hello_copy.txt", "docs/hello_renamed.txt");
  await store.renameOpts("docs/hello_copy2.txt", "docs/hello_renamed2.txt", {
    targetMode: "overwrite",
  });
  const renamedContent = await store.get("docs/hello_renamed.txt");
  assert.equal(renamedContent.toString("utf8"), "Hello Object Store!");

  // 9. List objects & ListWithOffset
  console.log("\n8. LIST objects & listWithOffset");
  const list = await store.list("docs");
  console.log("Listed objects under 'docs':");
  for (const item of list) {
    console.log(` - ${item.location} (${item.size} bytes, modified: ${item.lastModified})`);
  }

  const offsetList = await store.listWithOffset("docs", "docs/hello.txt");
  console.log("Listed with offset count:", offsetList.length);

  // 10. Delete & DeleteOpts object
  console.log("\n9. DELETE & deleteOpts object");
  await store.delete("docs/hello_renamed.txt");
  await store.deleteOpts("docs/hello_renamed2.txt", {});
  console.log("Deleted renamed files successfully.");

  // 11. Local File System & Parse URL Examples
  console.log("\n10. Local Store & parseUrl");
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "crud-example-"));
  try {
    const localStore = ObjectStore.local(tmpDir);
    await localStore.put("file.txt", Buffer.from("local content"));
    const localData = await localStore.get("file.txt");
    console.log("Local store file.txt:", localData.toString("utf8"));

    const urlParsed = parseUrl("memory://");
    await urlParsed.store.put("url_test.txt", Buffer.from("url content"));
    const urlData = await urlParsed.store.get("url_test.txt");
    console.log("Parsed URL store url_test.txt:", urlData.toString("utf8"));
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }

  console.log("\n=== All CRUD operations completed successfully! ===");
}

main().catch((err) => {
  console.error("Error running CRUD example:", err);
  process.exit(1);
});
