import { HttpClient, ClientOptions, HttpStore } from "../dist/index.js";

// Custom JS fetch adapter that uses globalThis.fetch
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
  const arrayBuffer = await res.arrayBuffer();
  return {
    status: res.status,
    headers: Object.fromEntries(res.headers),
    body: Buffer.from(arrayBuffer),
  };
};

async function main() {
  console.log("--- HTTP Client & HttpStore Example ---");

  // 1. Configure ClientOptions
  console.log("\n1. Configuring ClientOptions...");
  const options = new ClientOptions();
  options.withAllowHttp(true);
  options.withUserAgent("my-object-store-app/1.0");
  options.withTimeout(30);

  // 2. Create HttpClient and execute direct HTTP requests
  console.log("\n2. Executing direct request with HttpClient...");
  const client = new HttpClient(fetchAdapter, options);
  try {
    const response = await client.execute({
      url: "https://httpbin.org/get",
      method: "GET",
      headers: { "x-custom-header": "alloy-test" },
    });
    console.log("HttpClient Execute Status:", response.status);
    console.log("HttpClient Execute Body Preview:", response.body.toString("utf8").slice(0, 100));
  } catch (err) {
    console.log("HttpClient execute error (expected if offline or endpoint unreachable):", err);
  }

  // 3. Create HttpStore with the fetch adapter
  console.log("\n3. Using HttpStore with fetchAdapter...");
  const httpStore = HttpStore.withOptions(
    "https://example.com/webdav/",
    fetchAdapter,
    { allowHttp: true, retryMaxAttempts: 3 },
  );

  const store = httpStore.asObjectStore();
  console.log("HttpStore ObjectStore interface created successfully.");
  console.log("Supported methods on store:", Object.keys(Object.getPrototypeOf(store)));

  console.log("\n--- HTTP Client & HttpStore Example completed ---");
}

void main().catch(console.error);
