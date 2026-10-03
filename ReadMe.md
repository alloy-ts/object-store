# `@lib/object-store`

`@lib/object-store` provides high-performance Node.js native bindings for Apache Arrow's [`object_store`](https://crates.io/crates/object_store) crate via NAPI-RS.

It delivers a uniform API for interacting with cloud object storage services (S3, Azure Blob, Google Cloud Storage, WebDAV) and local file systems.

## Installation

```bash
npm install @lib/object-store
```

## Features

- **Uniform API**: Same code works across In-Memory, Local Filesystem, and Cloud URL endpoints.
- **High Performance**: Native Rust implementation using `object_store` crate.
- **Strongly Typed**: TypeScript interface definitions for all operations and options.
- **Feature Flags**: Introspect build-time enabled features via `getEnabledFeatures()`.

## Quick Start

```typescript
import { ObjectStore, getEnabledFeatures } from "@lib/object-store";

async function run() {
  console.log("Features:", getEnabledFeatures());

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
