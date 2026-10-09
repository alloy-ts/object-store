# Object Store

`@alloy-ts/object-store` is a high-performance native Rust addon for Node.js, providing TypeScript bindings via NAPI (Node-API) for Apache Arrow's [`object_store`](https://crates.io/crates/object_store).

It delivers a uniform API for interacting with blob-like (or document-like, graph-like) storage across different implementations, such as local database/filesystem, HTTP remote WebDAV, cloud object storage services (AWS S3, Azure Blob Storage, Google Cloud Storage, Cloudflare R2), etc.

## Installation

```bash
npm install @alloy-ts/object-store
```

## Features

- **Uniform API**: Same code works across In-Memory, Local Filesystem, and Cloud URL endpoints.
- **High Performance**: Native Rust implementation using `object_store` crate.
- **Strongly Typed**: TypeScript interface definitions for all operations and options.

## Quick Start

```typescript
import { ObjectStore } from "@alloy-ts/object-store";

async function run() {
  // Create an in-memory store
  const store = ObjectStore.createInMemory();

  // Write object
  await store.put("hello.txt", Buffer.from("Hello Object Store!"));

  // Read object
  const data = await store.get("hello.txt");
  console.log(data.toString()); // "Hello Object Store!"

  // Head metadata
  const meta = await store.head("hello.txt");
  console.log(`Size: ${meta.size} bytes`);
}

void run();
```

## Documentation

Full API documentation is available in [`docs/api.md`](./docs/api.md).

## License

MIT
