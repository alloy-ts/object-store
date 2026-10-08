import { ObjectStore } from "@alloy-ts/object-store";

async function main() {
  console.log("--- ObjectStore CRUD Example ---");

  // 1. Create store (In-Memory)
  const store = ObjectStore.createInMemory();

  // 2. CREATE / PUT
  console.log("\n1. PUT / CREATE:");
  const putRes = await store.put(
    "data/item1.json",
    Buffer.from(JSON.stringify({ name: "Widget", price: 100 })),
  );
  console.log("Put Result:", putRes);

  const putOptsRes = await store.putOpts(
    "data/item2.json",
    Buffer.from(JSON.stringify({ name: "Gadget", price: 200 })),
    { modeCreate: true },
  );
  console.log("PutOpts Result:", putOptsRes);

  // 3. READ / GET & HEAD
  console.log("\n2. READ / GET & HEAD:");
  const headMeta = await store.head("data/item1.json");
  console.log("Head Metadata:", headMeta);

  const data = await store.get("data/item1.json");
  console.log("Get Content:", data.toString());

  const getWithMeta = await store.getWithMeta("data/item1.json");
  console.log("GetWithMeta Result:", {
    content: getWithMeta.bytes.toString(),
    size: getWithMeta.meta.size,
  });

  const rangeData = await store.getOpts("data/item1.json", {
    range: { start: 0, end: 10 },
  });
  console.log("GetOpts Range Content:", rangeData.toString());

  // 4. LIST
  console.log("\n3. LIST:");
  const items = await store.list("data");
  console.log(
    "List items:",
    items.map((i) => i.location),
  );

  // 5. VECTORED READ / GET RANGES
  console.log("\n4. VECTORED READ:");
  const ranges = await store.getRanges("data/item1.json", [
    { start: 0, end: 5 },
    { start: 6, end: 11 },
  ]);
  console.log(
    "GetRanges Output:",
    ranges.map((r) => r.toString()),
  );

  // 6. COPY & RENAME
  console.log("\n5. COPY & RENAME:");
  await store.copy("data/item1.json", "data/item1_copy.json");
  console.log("Copied item1 to item1_copy");

  await store.rename("data/item2.json", "data/item2_renamed.json");
  console.log("Renamed item2 to item2_renamed");

  // 7. DELETE
  console.log("\n6. DELETE:");
  await store.delete("data/item1_copy.json");
  console.log("Deleted item1_copy.json");

  console.log("\n--- Example completed successfully ---");
}

void main().catch(console.error);
