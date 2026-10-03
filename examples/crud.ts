import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ObjectStore, parseUrl } from "../index.js";

async function main() {
  console.log("=== ObjectStore Core API CRUD Example ===");

  // 1. Create in-memory store
  const store = ObjectStore.memory();

  // 2. Put object (Create)
  console.log("\n1. PUT object");
  const putResult = await store.put("docs/hello.txt", Buffer.from("Hello Object Store!"));
  console.log("Put result:", putResult);

  // 3. Head object (Read Metadata)
  console.log("\n2. HEAD object");
  const meta = await store.head("docs/hello.txt");
  console.log("Object metadata:", meta);
  assert.equal(meta.location, "docs/hello.txt");
  assert.equal(meta.size, 19);

  // 4. Get object (Read Content)
  console.log("\n3. GET object");
  const content = await store.get("docs/hello.txt");
  console.log("Fetched content:", content.toString("utf8"));
  assert.equal(content.toString("utf8"), "Hello Object Store!");

  // 5. PutOpts object (Conditional / Opts)
  console.log("\n4. PUT with options");
  await store.putOpts("docs/overwrite.txt", Buffer.from("Overwritten content"), {
    mode: "overwrite",
  });

  // 6. GetOpts object (Partial Range / Range read)
  console.log("\n5. GET with options (Range)");
  const partial = await store.getOpts("docs/hello.txt", {
    rangeStart: 0,
    rangeEnd: 5,
  });
  console.log("Range GET (0..5):", partial.toString("utf8"));
  assert.equal(partial.toString("utf8"), "Hello");

  // 7. GetRanges (Vectored Read)
  console.log("\n6. GET ranges (Vectored Read)");
  const ranges = await store.getRanges("docs/hello.txt", [
    { start: 0, end: 5 },
    { start: 6, end: 12 },
  ]);
  console.log("Range 1 (0..5):", ranges[0]!.toString("utf8"));
  console.log("Range 2 (6..12):", ranges[1]!.toString("utf8"));

  // 8. Copy object
  console.log("\n7. COPY object");
  await store.copy("docs/hello.txt", "docs/hello_copy.txt");
  const copiedContent = await store.get("docs/hello_copy.txt");
  assert.equal(copiedContent.toString("utf8"), "Hello Object Store!");

  // 9. Rename / Move object
  console.log("\n8. RENAME object");
  await store.rename("docs/hello_copy.txt", "docs/hello_renamed.txt");
  const renamedContent = await store.get("docs/hello_renamed.txt");
  assert.equal(renamedContent.toString("utf8"), "Hello Object Store!");

  // 10. List objects
  console.log("\n9. LIST objects");
  const list = await store.list("docs");
  console.log("Listed objects under 'docs':");
  for (const item of list) {
    console.log(` - ${item.location} (${item.size} bytes, modified: ${item.lastModified})`);
  }

  // 11. Delete object
  console.log("\n10. DELETE object");
  await store.delete("docs/hello_renamed.txt");
  console.log("Deleted docs/hello_renamed.txt successfully.");

  // 12. Local File System & Parse URL Examples
  console.log("\n11. Local Store & parseUrl");
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
