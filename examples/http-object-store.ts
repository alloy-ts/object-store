import { ClientOptions, HttpClient, HttpStore } from "@alloy-ts/object-store";

// Custom JS fetch adapter
const fetchAdapter = async (req: {
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: Uint8Array;
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
  console.log("=== HTTP Client & ObjectStore Example ===\n");

  // 1. Configure ClientOptions
  const options = new ClientOptions();
  options.withAllowHttp(true);
  options.withTimeout(30);

  // 2. Create HttpClient with custom JS fetch adapter
  console.log("1. Executing direct HTTP request via HttpClient...");
  const client = new HttpClient(fetchAdapter, options);

  // Execute request
  const response = await client.execute({
    url: "https://httpbin.org/get",
    method: "GET",
  });

  console.log("Response Status:", response.status);
  console.log("Response Headers:", response.headers["content-type"]);
  console.log("Response Body (preview):", response.body.toString("utf8").slice(0, 150));

  // 3. Create HttpStore for higher-level ObjectStore operations over HTTP/WebDAV
  console.log("\n2. Initializing HttpStore over HTTP endpoint...");
  const httpStore = HttpStore.withOptions("https://httpbin.org/", fetchAdapter, {
    allowHttp: true,
  });
  const store = httpStore.asObjectStore();

  // Retrieve an object via HttpStore
  try {
    const data = await store.get("get");
    console.log("HttpStore GET 'get' status / length:", data.length, "bytes");
  } catch (err) {
    console.log("HttpStore operation result:", (err as Error).message);
  }

  console.log("\n=== Example completed successfully ===");
}

void main().catch(console.error);
