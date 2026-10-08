import { expect, test } from "vite-plus/test";
import { LocalFileSystem } from "../dist/index.js";
import { mkdtempSync, realpathSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

test("LocalFileSystem - prefix + put/get via ObjectStore", async () => {
  const dir = mkdtempSync(join(tmpdir(), "objstore-"));
  const fs = LocalFileSystem.newWithPrefix(dir);
  const store = fs.asObjectStore();

  await store.put("a/b.txt", Buffer.from("local"));
  const data = await store.get("a/b.txt");
  expect(data.toString()).toBe("local");

  // path_to_filesystem resolves to an absolute path under the prefix.
  // Compare against the canonical path: on macOS tmpdir() (/var/...) is a
  // symlink to /private/var/..., which `path_to_filesystem` resolves.
  const abs = fs.pathToFilesystem("a/b.txt");
  expect(abs.startsWith(realpathSync(dir))).toBe(true);
});

test("LocalFileSystem - builders return a new store", () => {
  const dir = mkdtempSync(join(tmpdir(), "objstore-"));
  const fs = LocalFileSystem.newWithPrefix(dir);
  const synced = fs.withFsync(true).withAutomaticCleanup(true);

  expect(typeof synced.asObjectStore().put).toBe("function");
});

test("LocalFileSystem - new() maps to the filesystem root", () => {
  const fs = new LocalFileSystem();
  expect(typeof fs.asObjectStore().put).toBe("function");
});
