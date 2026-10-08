import { expect, test } from "vite-plus/test";
import { ClientOptions, HttpClient, IoRuntime } from "../dist/index.js";
import http from "node:http";
import type { IncomingMessage, ServerResponse } from "node:http";

// The fetch adapter every request is delegated through (see HttpClient docs).
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

// Echo server matching the one in client.test.ts, just enough to prove a
// request actually crosses the dedicated IoRuntime.
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
    const body = await this.collect(req);
    res.statusCode = 200;
    res.end(body);
  }
}

// --- IoRuntime: construction -------------------------------------------------

test("IoRuntime - requires at least one worker thread", () => {
  expect(() => new IoRuntime(0)).toThrow();
});

test("IoRuntime - can be built with an explicit thread count", () => {
  // Must not throw for valid counts.
  expect(() => new IoRuntime(1)).not.toThrow();
  expect(() => new IoRuntime(4)).not.toThrow();
});

test("IoRuntime - defaults to tokio's worker count when omitted", () => {
  expect(() => new IoRuntime()).not.toThrow();
});

// --- HttpClient.withRuntime: requests driven by the dedicated runtime -------

test("HttpClient.withRuntime - executes a GET on the dedicated runtime", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const io = new IoRuntime(4);
    const options = new ClientOptions();
    options.withAllowHttp(true);
    const client = HttpClient.withRuntime(fetchAdapter, options, io);

    const res = await client.execute({
      url: `http://localhost:${port}/`,
      method: "GET",
    });
    expect(res.status).toBe(200);
  } finally {
    await server.close();
  }
});

test("HttpClient.withRuntime - round-trips a PUT body", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const io = new IoRuntime(2);
    const options = new ClientOptions();
    options.withAllowHttp(true);
    const client = HttpClient.withRuntime(fetchAdapter, options, io);

    const payload = Buffer.from("spawned-payload");
    const res = await client.execute({
      url: `http://localhost:${port}/objects/9`,
      method: "PUT",
      body: payload,
    });
    expect(res.status).toBe(200);
    expect(res.body.toString("utf8")).toBe("spawned-payload");
  } finally {
    await server.close();
  }
});

test("HttpClient.withRuntime - still refuses plain http without allow_http", async () => {
  const server = new EchoServer();
  const port = await server.start();
  try {
    const io = new IoRuntime(4);
    // No allow_http here, so the default https-only policy applies.
    const client = HttpClient.withRuntime(fetchAdapter, undefined, io);

    await expect(
      client.execute({ url: `http://localhost:${port}/`, method: "GET" }),
    ).rejects.toBeDefined();
  } finally {
    await server.close();
  }
});
