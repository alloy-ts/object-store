import { expect, test } from "vite-plus/test";
import { ObjectStore, ChunkedStore } from "../index.js";

// Deterministic payload so we can assert exact byte reassembly across chunk
// boundaries (byte i = i % 251 avoids a uniform stream).
const makeData = (n: number): Buffer => {
  const buf = Buffer.alloc(n);
  for (let i = 0; i < n; i++) buf[i] = i % 251;
  return buf;
};

// --- constructor contract ---------------------------------------------------

test("ChunkedStore - new rejects a zero chunk size", () => {
  const base = ObjectStore.createInMemory();
  expect(() => ChunkedStore.new(base, 0)).toThrow(/chunk_size must be >= 1/);
});

test("ChunkedStore - as_object_store exposes the full ObjectStore surface", () => {
  const base = ObjectStore.createInMemory();
  const store = ChunkedStore.new(base, 8).asObjectStore();
  expect(typeof store.put).toBe("function");
  expect(typeof store.get).toBe("function");
  expect(typeof store.head).toBe("function");
  expect(typeof store.list).toBe("function");
  expect(typeof store.delete).toBe("function");
  expect(typeof store.getRanges).toBe("function");
});

// --- chunk reassembly integrity ---------------------------------------------

// The napi `get` buffers the entire response, so chunking is transparent at the
// JS boundary; what matters is that bytes reassemble identically to the
// unchunked store regardless of where chunk boundaries fall.
for (const size of [1, 3, 7, 16, 64, 1000]) {
  test(`ChunkedStore - get reassembles content for chunk_size=${size}`, async () => {
    const base = ObjectStore.createInMemory();
    const data = makeData(1000);
    await base.put("obj.bin", data);

    const chunked = ChunkedStore.new(base, size).asObjectStore();
    const out = await chunked.get("obj.bin");
    expect(Buffer.isBuffer(out)).toBe(true);
    expect(out.length).toBe(data.length);
    expect(out.equals(data)).toBe(true);

    // Must match the unchunked read exactly.
    const direct = await base.get("obj.bin");
    expect(out.equals(direct)).toBe(true);
  });
}

test("ChunkedStore - small payloads still reassemble (incl. sub-chunk and empty)", async () => {
  const base = ObjectStore.createInMemory();

  const tiny = makeData(3); // smaller than every chunk size below
  await base.put("tiny.bin", tiny);
  for (const size of [1, 5, 100]) {
    const chunked = ChunkedStore.new(base, size).asObjectStore();
    expect((await chunked.get("tiny.bin")).equals(tiny)).toBe(true);
  }

  const empty = makeData(0);
  await base.put("empty.bin", empty);
  const chunked = ChunkedStore.new(base, 4).asObjectStore();
  expect((await chunked.get("empty.bin")).equals(empty)).toBe(true);
});

test("ChunkedStore - put through the wrapper then chunked get round-trips", async () => {
  const base = ObjectStore.createInMemory();
  const chunked = ChunkedStore.new(base, 5).asObjectStore();
  // put goes through the same inner store the chunked view wraps.
  await chunked.put("a/b.bin", makeData(123));
  const out = await chunked.get("a/b.bin");
  expect(out.equals(makeData(123))).toBe(true);
});

// --- other operations stay usable through the wrapper ------------------------

test("ChunkedStore - head/list/delete work via as_object_store", async () => {
  const base = ObjectStore.createInMemory();
  const chunked = ChunkedStore.new(base, 4).asObjectStore();

  await chunked.put("x.txt", Buffer.from("hello"));
  const meta = await chunked.head("x.txt");
  expect(meta.location).toBe("x.txt");
  expect(meta.size).toBe(5);

  const listed = await chunked.list();
  expect(listed.map((m) => m.location)).toContain("x.txt");

  await chunked.delete("x.txt");
  const after = await chunked.list();
  expect(after.map((m) => m.location)).not.toContain("x.txt");
});

test("ChunkedStore - get_ranges returns correct slices across chunks", async () => {
  const base = ObjectStore.createInMemory();
  const data = makeData(100);
  await base.put("r.bin", data);

  const chunked = ChunkedStore.new(base, 7).asObjectStore();
  const [a, b] = await chunked.getRanges("r.bin", [
    { start: 0, end: 7 }, // one full chunk
    { start: 7, end: 20 }, // spans into the next chunk
  ]);
  expect(a.equals(data.subarray(0, 7))).toBe(true);
  expect(b.equals(data.subarray(7, 20))).toBe(true);
});
