import { expect, test } from "vite-plus/test";
import { HttpStore, ClientOptions } from "../dist/index.js";
import http from "node:http";
import type { IncomingMessage, ServerResponse } from "node:http";

// The fetch adapter all HttpStore tests share: every request the store makes
// is delegated here, executed with the global fetch, and buffered before
// being handed back to Rust.
const fetchAdapter = async (request: {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: Uint8Array | null;
}): Promise<{ status: number; headers: Record<string, string>; body: Buffer }> => {
  const headers = { ...request.headers };
  delete headers["content-length"];
  delete headers["Content-Length"];
  const res = await fetch(request.url, {
    method: request.method,
    headers,
    body: request.body ?? undefined,
  });
  const body = await res.arrayBuffer();
  return {
    status: res.status,
    headers: Object.fromEntries(res.headers),
    body: Buffer.from(body),
  };
};

// Minimal in-process WebDAV server covering the exact subset object_store's
// HttpStore relies on: PUT, MKCOL, GET, HEAD, DELETE, COPY and PROPFIND.
// This lets the NAPI HttpStore be exercised end-to-end without an external
// server. It is intentionally tiny and not a general-purpose WebDAV server.
class WebDavServer {
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

  private keyOf(pathname: string): string {
    return decodeURIComponent((pathname ?? "/").split("?")[0]).replace(/^\/+/, "");
  }

  private async handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const key = this.keyOf(req.url);
    const method = req.method ?? "GET";
    const body = await this.collect(req);
    try {
      switch (method) {
        case "PUT":
          return this.put(key, body, res);
        case "MKCOL":
          return this.ok(res, 201);
        case "GET":
          return this.get(key, req, res, false);
        case "HEAD":
          return this.get(key, req, res, true);
        case "DELETE":
          return this.del(key, res);
        case "COPY":
          return this.copy(key, req, res);
        case "PROPFIND":
          return this.propfind(key, res);
        default:
          return this.ok(res, 405);
      }
    } catch (e) {
      res.statusCode = 500;
      res.end(String(e));
    }
  }

  private collect(req: IncomingMessage): Promise<Buffer> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = [];
      req.on("data", (c: Buffer) => chunks.push(c));
      req.on("end", () => resolve(Buffer.concat(chunks)));
      req.on("error", reject);
    });
  }

  private put(key: string, data: Buffer, res: ServerResponse): void {
    this.files.set(key, data);
    res.setHeader("ETag", `"${key}"`);
    res.statusCode = 201;
    res.end();
  }

  private get(key: string, req: IncomingMessage, res: ServerResponse, head: boolean): void {
    const data = this.files.get(key);
    if (!data) {
      res.statusCode = 404;
      res.end();
      return;
    }
    const lastModified = new Date(0).toUTCString();
    const etag = `"${key}"`;
    res.setHeader("ETag", etag);
    res.setHeader("Last-Modified", lastModified);
    res.setHeader("Accept-Ranges", "bytes");

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
      res.setHeader("Content-Length", data.length);
      res.end(head ? undefined : data);
    }
  }

  private del(key: string, res: ServerResponse): void {
    this.files.delete(key);
    res.statusCode = 204;
    res.end();
  }

  private copy(key: string, req: IncomingMessage, res: ServerResponse): void {
    const dest = req.headers["destination"];
    if (typeof dest !== "string") {
      res.statusCode = 400;
      res.end();
      return;
    }
    const destKey = this.keyOf(new URL(dest).pathname);
    const data = this.files.get(key);
    if (!data) {
      res.statusCode = 404;
      res.end();
      return;
    }
    this.files.set(destKey, data);
    res.statusCode = 201;
    res.end();
  }

  private dirsFor(keys: Iterable<string>): string[] {
    const dirs = new Set<string>();
    for (const k of keys) {
      const parts = k.split("/");
      for (let i = 1; i < parts.length; i++) dirs.add(parts.slice(0, i).join("/"));
    }
    return [...dirs];
  }

  private fileResponse(key: string, data: Buffer, base: string): string {
    const lastModified = new Date(0).toUTCString();
    return (
      `<response><href>${base}/${key}</href><propstat><prop>` +
      `<getlastmodified>${lastModified}</getlastmodified>` +
      `<getcontentlength>${data.length}</getcontentlength>` +
      `<resourcetype></resourcetype>` +
      `<getetag>"${key}"</getetag>` +
      `</prop><status>HTTP/1.1 200 OK</status></propstat></response>`
    );
  }

  private collectionResponse(key: string, base: string): string {
    const lastModified = new Date(0).toUTCString();
    return (
      `<response><href>${base}/${key}</href><propstat><prop>` +
      `<getlastmodified>${lastModified}</getlastmodified>` +
      `<resourcetype><collection/></resourcetype>` +
      `</prop><status>HTTP/1.1 200 OK</status></propstat></response>`
    );
  }

  private propfind(prefixKey: string, res: ServerResponse): void {
    const base = `http://localhost:${this.port}`;
    const files = [...this.files.entries()].filter(
      ([k]) => !prefixKey || k === prefixKey || k.startsWith(prefixKey + "/"),
    );
    const dirs = this.dirsFor(this.files.keys()).filter(
      (d) => !prefixKey || d === prefixKey || d.startsWith(prefixKey + "/"),
    );
    const body =
      `<?xml version="1.0" encoding="utf-8"?><multistatus>` +
      files.map(([k, v]) => this.fileResponse(k, v, base)).join("") +
      dirs.map((d) => this.collectionResponse(d, base)).join("") +
      `</multistatus>`;
    res.setHeader("Content-Type", 'application/xml; charset="utf-8"');
    res.statusCode = 207;
    res.end(body);
  }

  private ok(res: ServerResponse, code: number): void {
    res.statusCode = code;
    res.end();
  }
}

// --- construction / configuration (no server required) -----------------------

test("HttpStore - new / withOptions expose the ObjectStore surface", () => {
  const store = new HttpStore("https://example.com/root", fetchAdapter);
  const obj = store.asObjectStore();
  expect(typeof obj.put).toBe("function");
  expect(typeof obj.get).toBe("function");
  expect(typeof obj.list).toBe("function");
  expect(typeof obj.delete).toBe("function");

  const optStore = HttpStore.withOptions("http://example.com/root", fetchAdapter, {
    allowHttp: true,
    retryMaxAttempts: 3,
  });
  expect(typeof optStore.asObjectStore().get).toBe("function");
});

test("HttpStore - withClientOptions constructs store and executes requests", async () => {
  const server = new WebDavServer();
  const port = await server.start();
  try {
    const clientOptions = new ClientOptions();
    clientOptions.withAllowHttp(true);
    const store = HttpStore.withClientOptions(`http://localhost:${port}/`, fetchAdapter, clientOptions).asObjectStore();
    await store.put("client_opt.txt", Buffer.from("client_opt_data"));
    const data = await store.get("client_opt.txt");
    expect(data.toString()).toBe("client_opt_data");
  } finally {
    await server.close();
  }
});

test("HttpStore - new rejects a non-URL", () => {
  expect(() => new HttpStore("", fetchAdapter)).toThrow();
  expect(() => new HttpStore("not a url", fetchAdapter)).toThrow();
});

test("HttpStore - request against a dead port rejects", async () => {
  // Port 1 is never bound; a connection attempt must surface an error rather
  // than hanging or returning empty data.
  const store = new HttpStore("http://localhost:1/", fetchAdapter).asObjectStore();
  await expect(store.get("missing.txt")).rejects.toBeDefined();
});

// --- end-to-end against an embedded WebDAV server ----------------------------

test("HttpStore - put/get/head/getRanges/list/copy/delete round-trip", async () => {
  const server = new WebDavServer();
  const port = await server.start();
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, fetchAdapter, {
      allowHttp: true,
    }).asObjectStore();

    await store.put("a/b.txt", Buffer.from("hello"));

    const data = await store.get("a/b.txt");
    expect(data.toString()).toBe("hello");

    const meta = await store.head("a/b.txt");
    expect(meta.location).toBe("a/b.txt");
    expect(meta.size).toBe(5);

    const ranges = await store.getRanges("a/b.txt", [{ start: 1, end: 3 }]);
    expect(ranges.length).toBe(1);
    expect(ranges[0].toString()).toBe("el"); // "hello"[1..3]

    const listed = await store.list("a");
    expect(listed.map((m) => m.location)).toContain("a/b.txt");

    await store.copy("a/b.txt", "a/c.txt");
    const copied = await store.get("a/c.txt");
    expect(copied.toString()).toBe("hello");

    await store.delete("a/c.txt");
    const after = await store.list("a");
    expect(after.map((m) => m.location)).not.toContain("a/c.txt");
  } finally {
    await server.close();
  }
});

test("HttpStore - list_with_delimiter returns objects and common prefixes", async () => {
  const server = new WebDavServer();
  const port = await server.start();
  try {
    const store = HttpStore.withOptions(`http://localhost:${port}/`, fetchAdapter, {
      allowHttp: true,
    }).asObjectStore();
    await store.put("dir/x.txt", Buffer.from("x"));
    await store.put("dir/y.txt", Buffer.from("y"));
    await store.put("top.txt", Buffer.from("t"));

    const result = await store.listWithDelimiter();
    const locations = result.objects.map((o) => o.location);
    const prefixes = result.commonPrefixes;
    expect(locations).toContain("top.txt");
    expect(locations).toContain("dir/x.txt");
    expect(prefixes).toContain("dir");
  } finally {
    await server.close();
  }
});
