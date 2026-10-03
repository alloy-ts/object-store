import console from "node:console";
import { ObjectStore } from "../src/main.ts";

async function main() {
  const store = ObjectStore.inMemory();

  // 1. Put (Create / Update)
  console.log("--- Put Object ---");
  const putRes = await store.put("docs/readme.md", Buffer.from("# Readme Content"));
  console.log("Put result:", putRes);

  // 2. Put with Options (Conditional Put)
  console.log("--- Put with Options ---");
  const putOptsRes = await store.putOpts("docs/changelog.md", Buffer.from("# Changelog"), {
    mode: "create",
  });
  console.log("PutOpts result:", putOptsRes);

  // 3. Head (Metadata)
  console.log("--- Head Metadata ---");
  const meta = await store.head("docs/readme.md");
  console.log("Metadata:", meta);

  // 4. Get (Read)
  console.log("--- Get Object ---");
  const getRes = await store.get("docs/readme.md");
  console.log("Content:", getRes.bytes.toString());

  // 5. Get with Options (Conditional Fetch / Range)
  console.log("--- Get with Options ---");
  const getOptsRes = await store.getOpts("docs/readme.md", {
    range: { start: 0, length: 8 },
  });
  console.log("GetOpts range content:", getOptsRes.bytes.toString());

  // 6. Get Ranges (Vectored Read)
  console.log("--- Get Ranges ---");
  const ranges = await store.getRanges("docs/readme.md", [
    { start: 0, length: 1 },
    { start: 2, length: 6 },
  ]);
  console.log(
    "Ranges:",
    ranges.map((r) => r.toString()),
  );

  // 7. List
  console.log("--- List Objects ---");
  const list = await store.list("docs");
  console.log(
    "Listed objects:",
    list.map((o) => o.location),
  );

  // 8. Copy
  console.log("--- Copy Object ---");
  await store.copy("docs/readme.md", "docs/readme_backup.md");
  const copyMeta = await store.head("docs/readme_backup.md");
  console.log("Copy metadata:", copyMeta);

  // 9. Rename
  console.log("--- Rename Object ---");
  await store.rename("docs/readme_backup.md", "docs/readme_v2.md");
  const renameMeta = await store.head("docs/readme_v2.md");
  console.log("Rename metadata:", renameMeta);

  // 10. Delete
  console.log("--- Delete Object ---");
  await store.delete("docs/readme_v2.md");
  const listAfterDelete = await store.list("docs");
  console.log(
    "Objects after delete:",
    listAfterDelete.map((o) => o.location),
  );
}

void main();
