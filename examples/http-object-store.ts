import { HttpStore } from "../index.js";

// Custom JS fetch adapter
const fetchAdapter = async (req: {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: Uint8Array | null;
}) => {
  const res = await fetch(req.url, {
    method: req.method,
    headers: req.headers,
    body: req.body ?? undefined,
  });
  return {
    status: res.status,
    headers: Object.fromEntries(res.headers),
    body: Buffer.from(await res.arrayBuffer()),
  };
};

async function main() {
  console.log("--- HttpStore Example ---");

  // Create an HttpStore for a base URL using the fetch adapter
  const store = HttpStore.withOptions("https://httpbin.org/", fetchAdapter, {
    allowHttp: true,
    retryMaxAttempts: 2,
  }).asObjectStore();

  console.log("HttpStore initialized successfully.");
  console.log("HttpStore ObjectStore instance created:", store);
}

void main().catch(console.error);
