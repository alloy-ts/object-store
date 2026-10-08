import { LocalFileSystem } from "../index.js";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

async function main() {
  console.log("--- LocalFileSystem Object Store Example ---");

  const tmpDir = mkdtempSync(join(tmpdir(), "fs-objstore-example-"));

  try {
    // Create LocalFileSystem with a root prefix
    const fsStore = LocalFileSystem.newWithPrefix(tmpDir);
    const store = fsStore.asObjectStore();

    // 1. Write file
    await store.put("documents/hello.txt", Buffer.from("Hello from LocalFileSystem!"));
    console.log("Written file to LocalFileSystem");

    // 2. Read file
    const content = await store.get("documents/hello.txt");
    console.log("Retrieved content:", content.toString());

    // 3. Inspect metadata & filesystem path
    const meta = await store.head("documents/hello.txt");
    console.log("Metadata:", meta);

    const absPath = fsStore.pathToFilesystem("documents/hello.txt");
    console.log("Absolute path on disk:", absPath);

    // 4. List files
    const listed = await store.list("documents");
    console.log("Listed files:", listed.map((m) => m.location));

    // 5. Clean up file via store
    await store.delete("documents/hello.txt");
    console.log("Deleted file from store");
  } finally {
    rmSync(tmpDir, { recursive: true, force: true });
  }

  console.log("--- LocalFileSystem Example Completed ---");
}

void main().catch(console.error);
