import { expect, test } from "vite-plus/test";
import { HttpStore } from "../dist/index.js";
import http from "node:http";
import type { IncomingMessage, ServerResponse } from "node:http";

// A fetch adapter that records every request descriptor the store hands to JS
// (the subject of src/fetch.rs) and delegates the real I/O to a tiny in-memory
// server. This lets us assert both the *request* shape (method/url/headers/body)
// and the *response* mapping (status/headers/body) without an external server.
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
// PROPFIND/list is intentionally omitted here — it is covered by http.test.ts.
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
  }).asObjectStore();
  await expect(store.get("x.txt")).rejects.toThrow(/adapter exploded/);
});
