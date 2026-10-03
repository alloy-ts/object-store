# @lib/object-store

High-performance Node.js native bindings for the Apache Arrow [`object_store`](https://docs.rs/object_store/latest/object_store/) Rust crate via NAPI-RS.

`@lib/object-store` provides a uniform, asynchronous TypeScript/JavaScript interface for interacting with cloud object storage services (S3, GCS, Azure, WebDAV) and local filesystems.

---

## Features

- **Unified Async API**: Single API for in-memory, local filesystem, and cloud storage providers.
- **Vectored Read & Write**: High-throughput parallel range requests (`getRanges`) and multipart uploads.
- **Conditional Reads & Writes**: Support for OCC (Optimistic Concurrency Control) transactions using `ifMatch`, `ifNoneMatch`, `PutMode`, and conditional updates.
- **URL Configuration**: Create stores via URL strings (e.g. `memory://`, `file:///path`, `s3://bucket/path`) with key-value option maps.
- **Strongly Typed**: Built-in TypeScript definitions generated directly from Rust NAPI attributes.

---

## Quick Start

```ts
import { ObjectStore } from "@lib/object-store";

async function example() {
  // Create an in-memory object store
  const store = ObjectStore.memory();

  // Put an object
  await store.put("hello.txt", Buffer.from("Hello Object Store!"));

  // Fetch object content
  const data = await store.get("hello.txt");
  console.log(data.toString("utf8")); // "Hello Object Store!"

  // Head object metadata
  const meta = await store.head("hello.txt");
  console.log(`Size: ${meta.size} bytes, modified: ${meta.lastModified}`);

  // List objects under a prefix
  const items = await store.list("hello");
  console.log(`Found ${items.length} items.`);
}

example();
```

---

## Detailed Example & Documentation

- See [`examples/crud.ts`](./examples/crud.ts) for a full runnable CRUD example showcasing all core methods.
- See [`docs/api.md`](./docs/api.md) for the complete API reference and type definitions.

---

## Development

- Install dependencies:

```bash
npm install
```

- Run unit tests:

```bash
npm test
```

- Run CRUD example:

```bash
node examples/crud.ts
```

- Build native binary:

```bash
npm run build
```

- Linting and Formatting:

```bash
npm run fmt
npm run lint
npm run check
```
