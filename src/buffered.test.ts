import { expect, test } from "vite-plus/test";
import { BufReader, BufWriter, InMemory } from "../../index.js";

test("BufReader - reads sequentially and supports seek", async () => {
  const store = new InMemory();
  const data = Buffer.alloc(4096, 7);
  await store.put("reader.txt", data);

  const meta = await store.head("reader.txt");
  const reader = BufReader.create(store, meta, { capacity: 256 });

  expect(reader.size()).toBe(4096);

  let out = Buffer.alloc(0);
  while (true) {
    const chunk = await reader.read(100);
    if (chunk.length === 0) break;
    out = Buffer.concat([out, chunk]);
  }
  expect(out.length).toBe(4096);
  expect(out.every((b) => b === 7)).toBe(true);

  // Seek back to 10 and inspect the buffered window.
  await reader.seek({ kind: "start", offset: 10n });
  expect(await reader.streamPosition()).toBe(10n);

  const buf = await reader.fillBuf();
  expect(buf.length).toBe(256);
  expect(buf[0]).toBe(7);

  // Seeking beyond the end returns no data.
  await reader.seek({ kind: "end", offset: 0n });
  expect((await reader.fillBuf()).length).toBe(0);
});

test("BufWriter - buffers then flushes via shutdown", async () => {
  const store = new InMemory();
  const writer = BufWriter.create(store, "writer.txt", { capacity: 64 });
  await writer.write(Buffer.from("hello "));
  await writer.write(Buffer.from("world"));
  await writer.shutdown();

  const got = await store.get("writer.txt");
  expect(got.toString()).toBe("hello world");
});

test("BufWriter - put path writes bytes", async () => {
  const store = new InMemory();
  const writer = BufWriter.create(store, "put.txt", { capacity: 64 });
  await writer.put(Buffer.from([1, 2, 3, 4]));
  await writer.shutdown();

  const got = await store.get("put.txt");
  expect([...got]).toEqual([1, 2, 3, 4]);
});

test("BufWriter - abort discards buffered data", async () => {
  const store = new InMemory();
  const writer = BufWriter.create(store, "abort.txt", { capacity: 64 });
  await writer.write(Buffer.from("data"));
  await writer.abort();

  const list = await store.list();
  expect(list.map((m) => m.location)).not.toContain("abort.txt");
});
