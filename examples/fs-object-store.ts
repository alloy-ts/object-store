import { LocalFileSystem } from "../dist/index.js";
import * as path from "node:path";
import * as os from "node:os";
import * as fs from "node:fs/promises";

async function main() {
  console.log("--- LocalFileSystem Example ---");

  // Create a temporary directory for local file store
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "object-store-fs-"));
  console.log("Working Directory:", tmpDir);

  try {
    // 1. Create LocalFileSystem instance
    const fsStore = new LocalFileSystem(tmpDir);
    const store = fsStore.asObjectStore();

    // 2. Put / Write a file
    console.log("\n1. Writing files...");
    await store.put("hello.txt", Buffer.from("Hello from LocalFileSystem!"));
    await store.put("folder/nested.json", Buffer.from(JSON.stringify({ key: "value" })));

    // 3. Read / Get content
    console.log("\n2. Reading files...");
    const content = await store.get("hello.txt");
    console.log("hello.txt content:", content.toString("utf8"));

    const meta = await store.head("hello.txt");
    console.log("hello.txt metadata:", {
      location: meta.location,
      size: meta.size,
      lastModified: meta.lastModified,
    });

    // 4. List files
    console.log("\n3. Listing files...");
    const files = await store.list();
    console.log(
      "Listed files:",
      files.map((f) => f.location),
    );

    // 5. Copy & Delete
    console.log("\n4. Copy and Delete...");
    await store.copy("hello.txt", "hello_backup.txt");
    const copied = await store.get("hello_backup.txt");
    console.log("Copied file content:", copied.toString("utf8"));

    await store.delete("hello_backup.txt");
    console.log("Deleted hello_backup.txt");
  } finally {
    // Cleanup temporary directory
    await fs.rm(tmpDir, { recursive: true, force: true });
    console.log("\nCleaned up temp directory.");
  }

  console.log("\n--- LocalFileSystem Example completed ---");
}

void main().catch(console.error);
