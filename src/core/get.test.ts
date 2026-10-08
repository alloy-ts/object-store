import { expect, test } from "vite-plus/test";
import { HttpStore, ObjectStore } from "../../dist/index.js";
import http from "node:http";
import type { IncomingMessage, ServerResponse } from "node:http";

test("Get - get, getWithMeta, getOpts", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("hello.txt", Buffer.from("Hello World!"));

  const data = await store.get("hello.txt");
  expect(data.toString()).toBe("Hello World!");

  const res = await store.getWithMeta("hello.txt");
  expect(res.bytes.toString()).toBe("Hello World!");
  expect(res.meta.location).toBe("hello.txt");
  expect(res.meta.size).toBe(12);

  const rangeData = await store.get("hello.txt", {
    range: { start: 0, end: 5 },
  });
  expect(rangeData.toString()).toBe("Hello");

  const rangeDataOpts = await store.getOpts("hello.txt", {
    range: { start: 0, end: 5 },
  });
  expect(rangeDataOpts.toString()).toBe("Hello");
});

test("Get - getWithMeta reports the served range and attributes", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("ranged.txt", Buffer.from("Hello World!"), {
    attributes: { "content-type": "text/plain" },
  });

  const whole = await store.getWithMeta("ranged.txt");
  expect(whole.range).toEqual({ start: 0, end: 12 });
  expect(whole.attributes["content-type"]).toBe("text/plain");

  const partial = await store.getWithMeta("ranged.txt", { range: { start: 6, end: 11 } });
  expect(partial.bytes.toString()).toBe("World");
  expect(partial.range).toEqual({ start: 6, end: 11 });
  // Attributes describe the object, so they are reported for ranged reads too.
  expect(partial.attributes["content-type"]).toBe("text/plain");
});

test("Get - getting non-existent key rejects with error", async () => {
  const store = ObjectStore.createInMemory();
  await expect(store.get("non-existent.txt")).rejects.toBeDefined();
});

test("Get - range options support offset and suffix reads", async () => {
  const store = ObjectStore.createInMemory();
  await store.put("offset.txt", Buffer.from("0123456789"));

  const offsetRead = await store.get("offset.txt", {
    range: { offset: 5 },
  });
  expect(offsetRead.toString()).toBe("56789");

  const suffixRead = await store.get("offset.txt", {
    range: { suffix: 4 },
  });
  expect(suffixRead.toString()).toBe("6789");
});

// A fetch adapter that records every request descriptor the store hands to JS
// and delegates the real I/O to a tiny in-memory server.
type RecordedRequest = {
  url: string;
  method: string;
  headers: Record<string, string>;
  body: Uint8Array | null;
};

const makeFetchAdapter = (server: InMemoryServer) => {
  const calls: RecordedRequest[] = [];
  const adapter = async (request: {
    url: string;
    method: string;
    headers: Record<string, string>;
    body?: Uint8Array | null;
  }): Promise<{ status: number; headers: Record<string, string>; body: Buffer }> => {
    calls.push({
      url: request.url,
      method: request.method,
      headers: request.headers,
      body: request.body ?? null,
    });
    const res = await fetch(request.url, {
      method: request.method,
      headers: request.headers,
      body: request.body ?? undefined,
    });
    const buf = Buffer.from(await res.arrayBuffer());
    return {
      status: res.status,
      headers: Object.fromEntries(res.headers),
      body: buf,
    };
  };
  return { adapter, calls };
};

// Minimal in-memory WebDAV surface: PUT, GET, HEAD, DELETE (ranges supported).
class InMemoryServer {
  private server!: http.Server;
  private files = new Map<string, Buffer>();
  private port = 0;

  async start(): Promise<number> {
    this.server = http.createServer((req, res) => void this.handle(req, res));
    await new Promise<void>((resolve) => this.server.listen(0, resolve));
    this.port = (this.server.address() as { port: number }).port;
    return this.port;
  }

  close(): Promise<void> {
    return new Promise((resolve) => this.server.close(() => resolve()));
  }

  private key(url: string): string {
    return decodeURIComponent(new URL(url, "http://localhost").pathname).replace(/^\/+/, "");
  }

  private collect(req: IncomingMessage): Promise<Buffer> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = [];
      req.on("data", (c: Buffer) => chunks.push(c));
      req.on("end", () => resolve(Buffer.concat(chunks)));
      req.on("error", reject);
    });
  }

  private async handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const key = this.key(req.url ?? "/");
    const method = req.method ?? "GET";
    try {
      switch (method) {
        case "PUT":
          this.files.set(key, await this.collect(req));
          res.statusCode = 201;
          res.end();
          return;
        case "GET":
          return this.read(key, req, res, false);
        case "HEAD":
          return this.read(key, req, res, true);
        case "DELETE":
          this.files.delete(key);
          res.statusCode = 204;
          res.end();
          return;
        default:
          res.statusCode = 405;
          res.end();
      }
    } catch (e) {
      res.statusCode = 500;
      res.end(String(e));
    }
  }

  private read(key: string, req: IncomingMessage, res: ServerResponse, head: boolean): void {
    const data = this.files.get(key);
    if (!data) {
      res.statusCode = 404;
      res.end();
      return;
    }
    res.setHeader("Content-Length", data.length);
    const range = req.headers["range"];
    const m = typeof range === "string" ? /^bytes=(\d+)-(\d*)$/.exec(range) : null;
    if (m) {
      const start = parseInt(m[1], 10);
      const end = m[2] ? parseInt(m[2], 10) : data.length - 1;
      const slice = data.subarray(start, end + 1);
      res.statusCode = 206;
      res.setHeader("Content-Range", `bytes ${start}-${end}/${data.length}`);
      res.setHeader("Content-Length", slice.length);
      res.end(head ? undefined : slice);
    } else {
      res.statusCode = 200;
      res.end(head ? undefined : data);
    }
  }
}

// --- request descriptor handed to the adapter -------------------------------

test("Fetch - adapter receives the full URL and correct method per op", async () => {
  const server = new InMemoryServer();
  const port = await server.start();
  const { adapter, calls } = makeFetchAdapter(server);
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, adapter, { allowHttp: true }).asObjectStore();
    await store.put("a/b.txt", Buffer.from("hello"));
    await store.get("a/b.txt");
    await store.head("a/b.txt");
    await store.delete("a/b.txt");

    const methods = calls.map((c) => c.method);
    expect(methods).toContain("PUT");
    expect(methods).toContain("GET");
    expect(methods).toContain("HEAD");
    expect(methods).toContain("DELETE");

    for (const call of calls) {
      expect(call.url.startsWith(`http://localhost:${port}/`)).toBe(true);
    }
  } finally {
    await server.close();
  }
});

test("Fetch - bodiless requests pass body: null, PUT passes a Uint8Array", async () => {
  const server = new InMemoryServer();
  const port = await server.start();
  const { adapter, calls } = makeFetchAdapter(server);
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, adapter, { allowHttp: true }).asObjectStore();
    await store.put("a/b.txt", Buffer.from("hello"));
    await store.get("a/b.txt");

    const put = calls.find((c) => c.method === "PUT")!;
    expect(put.body).toBeInstanceOf(Uint8Array);
    expect(Buffer.from(put.body as Uint8Array).toString()).toBe("hello");

    const get = calls.find((c) => c.method === "GET")!;
    expect(get.body).toBeNull();
  } finally {
    await server.close();
  }
});

test("Fetch - request headers are forwarded as a plain string map", async () => {
  const server = new InMemoryServer();
  const port = await server.start();
  const { adapter, calls } = makeFetchAdapter(server);
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, adapter, { allowHttp: true }).asObjectStore();
    await store.put("a/b.txt", Buffer.from("hello"));
    await store.getRanges("a/b.txt", [{ start: 0, end: 1 }]);

    const get = calls.find((c) => c.method === "GET")!;
    expect(typeof get.headers).toBe("object");
    // object_store sends a Range header for ranged reads.
    expect(get.headers["range"]).toBeDefined();
  } finally {
    await server.close();
  }
});

// --- response descriptor mapped back from the adapter -----------------------

test("Fetch - response status/body propagate to store reads", async () => {
  const server = new InMemoryServer();
  const port = await server.start();
  const { adapter } = makeFetchAdapter(server);
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, adapter, { allowHttp: true }).asObjectStore();
    await store.put("a/b.txt", Buffer.from("hello"));
    const data = await store.get("a/b.txt");
    expect(data.toString()).toBe("hello");

    const meta = await store.head("a/b.txt");
    expect(meta.size).toBe(5);
  } finally {
    await server.close();
  }
});

test("Fetch - 404 response surfaces as a rejected get", async () => {
  const server = new InMemoryServer();
  const port = await server.start();
  const { adapter } = makeFetchAdapter(server);
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, adapter, { allowHttp: true }).asObjectStore();
    await expect(store.get("missing.txt")).rejects.toBeDefined();
  } finally {
    await server.close();
  }
});

test("Fetch - a throwing adapter rejects the in-flight request", async () => {
  const boom = async (): Promise<never> => {
    throw new Error("adapter exploded");
  };
  const store = HttpStore.withOptions("http://localhost:1/", boom, {
    allowHttp: true,
    retryMaxAttempts: 0,
  }).asObjectStore();
  await expect(store.get("x.txt")).rejects.toThrow(/adapter exploded/);
});
