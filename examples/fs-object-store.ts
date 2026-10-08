import { LocalFileSystem, ObjectStore } from "@alloy-ts/object-store";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

async function main() {
  console.log("=== Local Filesystem ObjectStore Example ===");

  // Create a temporary directory for this example
  const tempDir = mkdtempSync(join(tmpdir(), "fs-object-store-example-"));
  console.log(`Working directory: ${tempDir}\n`);

  try {
    // Method 1: Using LocalFileSystem builder
    const fsStore = LocalFileSystem.newWithPrefix(tempDir);
    const store: ObjectStore = fsStore.asObjectStore();

    // Alternatively, Method 2: ObjectStore.createLocal(tempDir)
    // const store = ObjectStore.createLocal(tempDir);

    // 1. Write files (PUT)
    console.log("1. Writing files to local store...");
    await store.put("documents/file1.txt", Buffer.from("Hello from LocalFileSystem!"));
    await store.put(
      "documents/data.json",
      Buffer.from(JSON.stringify({ project: "@alloy-ts/object-store", type: "filesystem" })),
    );
    console.log("   Files written successfully.");

    // 2. Read files (GET)
    console.log("\n2. Reading file content...");
    const textData = await store.get("documents/file1.txt");
    console.log("   Content of 'documents/file1.txt':", textData.toString());

    const jsonData = await store.get("documents/data.json");
    console.log("   Content of 'documents/data.json':", jsonData.toString());

    // 3. Inspect metadata (HEAD)
    console.log("\n3. Inspecting metadata...");
    const meta = await store.head("documents/file1.txt");
    console.log("   Location:", meta.location);
    console.log("   Size:", meta.size, "bytes");
    console.log("   Last Modified:", new Date(meta.lastModified).toISOString());

    // 4. Resolve absolute path on disk
    const absPath = fsStore.pathToFilesystem("documents/file1.txt");
    console.log("\n4. Resolved filesystem path:", absPath);

    // 5. List objects in directory
    console.log("\n5. Listing objects with prefix 'documents'...");
    const items = await store.list("documents");
    for (const item of items) {
      console.log(`   - ${item.location} (${item.size} bytes)`);
    }

    // 6. Delete an object
    console.log("\n6. Deleting 'documents/file1.txt'...");
    await store.delete("documents/file1.txt");
    console.log("   File deleted.");

    const remaining = await store.list("documents");
    console.log("   Remaining files:", remaining.map((i) => i.location));

    console.log("\n=== Example completed successfully ===");
  } finally {
    // Cleanup temporary directory
    rmSync(tempDir, { recursive: true, force: true });
  }
}

void main().catch(console.error);
