import { expect, test } from "vite-plus/test";
import { ClientOptions, HttpClient } from "../dist/index.js";
import http from "node:http";
import type { IncomingMessage, ServerResponse } from "node:http";

// The fetch adapter every HttpClient test shares: each request is delegated to
// the global fetch and buffered before being handed back to Rust, mirroring the
// contract documented on `HttpClient`.
const fetchAdapter = async (request: {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: Uint8Array | null;
}): Promise<{ status: number; headers: Record<string, string>; body: Buffer }> => {
  const res = await fetch(request.url, {
    method: request.method,
    headers: request.headers,
    body: request.body ?? undefined,
  });
  const body = await res.arrayBuffer();
  return {
    status: res.status,
    headers: Object.fromEntries(res.headers),
    body: Buffer.from(body),
  };
};

// A tiny in-process echo server. Every normal request echoes the request body
// back verbatim and reflects each incoming header as `x-echo-<name>` so the
// client can assert on what the transport actually sent. Two special routes
// exercise non-2xx statuses and request timeouts.
class EchoServer {
  private server!: http.Server;
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

  private collect(req: IncomingMessage): Promise<Buffer> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = [];
      req.on("data", (c: Buffer) => chunks.push(c));
      req.on("end", () => resolve(Buffer.concat(chunks)));
      req.on("error", reject);
    });
  }

  private async handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const pathname = new URL(req.url ?? "/", `http://localhost:${this.port}`).pathname;

    if (pathname === "/slow") {
      // Exceeds the short client timeout configured in the timeout test.
      setTimeout(() => {
        res.statusCode = 200;
        res.end("slow");
      }, 1000);
      return;
    }

    if (pathname === "/missing") {
      res.statusCode = 404;
      res.end("nope");
      return;
    }

    const body = await this.collect(req);
    // Reflect every received header so the caller can verify what was sent.
    for (const [name, value] of Object.entries(req.headers)) {
      res.setHeader(
        `x-echo-${name.toLowerCase()}`,
        Array.isArray(value) ? value.join(",") : value,
      );
    }
    res.statusCode = 200;
    res.end(body);
  }
}

// --- ClientOptions: configuration surface (no server required) -------------

test("ClientOptions - defaults and config round-trips", () => {
  const options = new ClientOptions();
  expect(options.getConfigValue("allow_http")).toBeNull();

  options.withAllowHttp(true);
  expect(options.getConfigValue("allow_http")).toBe("true");

  // with_config / get_config_value round-trip arbitrary keys.
  options.withConfig("pool_idle_timeout", "60");
  expect(options.getConfigValue("pool_idle_timeout")).toBe("60");
});

test("ClientOptions - user agent is stored and readable", () => {
  const options = new ClientOptions();
  options.withUserAgent("my-app/1.0");
  expect(options.getConfigValue("user_agent")).toBe("my-app/1.0");
});

test("ClientOptions - default headers are recorded and surfaced", () => {
  const options = new ClientOptions();
  options.withDefaultHeaders({ "x-tenant": "acme" });
  const headers = options.getDefaultHeaders();
  expect(headers).not.toBeNull();
  expect(headers?.["x-tenant"]).toBe("acme");
});

test("ClientOptions - timeout can be set and disabled", () => {
  const options = new ClientOptions();
  options.withTimeout(5);
  expect(options.getConfigValue("timeout")).toBe("5s");
  options.withTimeoutDisabled();
  expect(options.getConfigValue("timeout")).toBeNull();
});

test("ClientOptions - content type resolution honours suffix rules", () => {
  const options = new ClientOptions();
  options.withDefaultContentType("application/octet-stream");
  options.withContentTypeForSuffix("json", "application/json");
  expect(options.getContentType("a/b.json")).toBe("application/json");
  expect(options.getContentType("a/b.bin")).toBe("application/octet-stream");
});

test("ClientOptions - clone is an independent copy", () => {
  const options = new ClientOptions();
  options.withAllowHttp(true);
  const copy = options.clone();
  copy.withAllowHttp(false);
  // Mutating the clone must not affect the original.
  expect(options.getConfigValue("allow_http")).toBe("true");
  expect(copy.getConfigValue("allow_http")).toBe("false");
});

// --- HttpClient: construction -------------------------------------------------

test("HttpClient - can be constructed with just a fetch adapter", () => {
  const client = new HttpClient(fetchAdapter);
  expect(typeof client.execute).toBe("function");
});

test("HttpClient - can be constructed with options", () => {
  const options = new ClientOptions();
  options.withAllowHttp(true);
  const client = new HttpClient(fetchAdapter, options);
  expect(typeof client.execute).toBe("function");
});

// --- HttpClient: end-to-end against the echo server -------------------------

test("HttpClient - executes a GET and buffers the response", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const client = new HttpClient(fetchAdapter, (() => {
      const o = new ClientOptions();
      o.withAllowHttp(true);
      return o;
    })());
    const res = await client.execute({
      url: `http://localhost:${port}/`,
      method: "GET",
    });
    expect(res.status).toBe(200);
    expect(res.body.toString("utf8")).toBe("");
  } finally {
    await server.close();
  }
});

test("HttpClient - refuses plain http unless allow_http is enabled", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const plain = new HttpClient(fetchAdapter);
    await expect(
      plain.execute({ url: `http://localhost:${port}/`, method: "GET" }),
    ).rejects.toBeDefined();

    const allowed = new HttpClient(fetchAdapter, (() => {
      const o = new ClientOptions();
      o.withAllowHttp(true);
      return o;
    })());
    const res = await allowed.execute({
      url: `http://localhost:${port}/`,
      method: "GET",
    });
    expect(res.status).toBe(200);
  } finally {
    await server.close();
  }
});

test("HttpClient - PUT round-trips the request body", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const client = new HttpClient(fetchAdapter, (() => {
      const o = new ClientOptions();
      o.withAllowHttp(true);
      return o;
    })());

    const payload = Buffer.from("object-store-payload");
    const res = await client.execute({
      url: `http://localhost:${port}/objects/1`,
      method: "PUT",
      body: payload,
    });
    expect(res.status).toBe(200);
    expect(res.body.toString("utf8")).toBe("object-store-payload");
  } finally {
    await server.close();
  }
});

test("HttpClient - sends the User-Agent configured via ClientOptions", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const options = new ClientOptions();
    options.withAllowHttp(true);
    options.withUserAgent("probe/9.9");
    const client = new HttpClient(fetchAdapter, options);

    const res = await client.execute({
      url: `http://localhost:${port}/`,
      method: "GET",
    });
    expect(res.headers["x-echo-user-agent"]).toBe("probe/9.9");
  } finally {
    await server.close();
  }
});

test("HttpClient - merges default headers, per-request headers win", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const options = new ClientOptions();
    options.withAllowHttp(true);
    options.withDefaultHeaders({ "x-tenant": "acme" });
    const client = new HttpClient(fetchAdapter, options);

    // No per-request override: the default header is sent.
    const res = await client.execute({
      url: `http://localhost:${port}/`,
      method: "GET",
    });
    expect(res.headers["x-echo-x-tenant"]).toBe("acme");

    // Per-request header overrides the default.
    const res2 = await client.execute({
      url: `http://localhost:${port}/`,
      method: "GET",
      headers: { "x-tenant": "override" },
    });
    expect(res2.headers["x-echo-x-tenant"]).toBe("override");
  } finally {
    await server.close();
  }
});

test("HttpClient - non-2xx status is returned, not thrown", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const client = new HttpClient(fetchAdapter, (() => {
      const o = new ClientOptions();
      o.withAllowHttp(true);
      return o;
    })());

    const res = await client.execute({
      url: `http://localhost:${port}/missing`,
      method: "GET",
    });
    expect(res.status).toBe(404);
    expect(res.body.toString("utf8")).toBe("nope");
  } finally {
    await server.close();
  }
});

test("HttpClient - honours the request timeout", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const options = new ClientOptions();
    options.withAllowHttp(true);
    options.withTimeout(0.05);
    const client = new HttpClient(fetchAdapter, options);

    await expect(
      client.execute({ url: `http://localhost:${port}/slow`, method: "GET" }),
    ).rejects.toBeDefined();
  } finally {
    await server.close();
  }
});
