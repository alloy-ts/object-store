# @lib/object-store

High-performance Node.js / TypeScript bindings to Apache Arrow's `object_store` Rust crate via NAPI-RS.

Provides a uniform, async API for interacting with local files, in-memory storage, Amazon S3, Google Cloud Storage, Azure Blob Storage, and HTTP servers.

## Features

- **Universal API**: Uniform interface across memory, local filesystem, and cloud storage providers.
- **Strongly Typed**: Auto-generated TypeScript definitions with full type safety (`GetOptions`, `PutOptions`, `CopyOptions`, `RenameOptions`, `ObjectMeta`, `GetResult`, etc.).
- **High Performance**: Native Rust core compiled to platform-specific binaries using NAPI-RS.

## Installation

```bash
npm install @lib/object-store
```

## Quick Start

```ts
import { ObjectStore, parseUrl } from "@lib/object-store";

// Create in-memory store
const store = ObjectStore.inMemory();

// Put an object
await store.put("hello.txt", Buffer.from("Hello, Object Store!"));

// Fetch object
const result = await store.get("hello.txt");
console.log(result.bytes.toString()); // "Hello, Object Store!"

// Head metadata
const meta = await store.head("hello.txt");
console.log(`Size: ${meta.size} bytes, ETag: ${meta.eTag}`);

// Vectored Read (Range Request)
const ranges = await store.getRanges("hello.txt", [
  { start: 0, length: 5 },
  { start: 7, length: 6 },
]);
console.log(ranges[0].toString()); // "Hello"

// Parse Store URL
const parsed = parseUrl("memory://path/to/resource");
await parsed.store.put(parsed.path, Buffer.from("data"));
```

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
vp install
```

- Build native module and TypeScript package:

```bash
npm run build
```

- Run unit tests:

```bash
npm test
```

- Code check & linting:

```bash
npm run check
npm run lint
```
