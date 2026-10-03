# ObjectStore Core API Reference

The `@lib/object-store` package provides high-performance Node.js / TypeScript bindings to the Rust `object_store` crate.

## Factory Functions & Constructors

### `ObjectStore.inMemory(): ObjectStore`

Creates an in-memory object store instance.

### `ObjectStore.local(path: string): ObjectStore`

Creates a local filesystem object store rooted at `path`.

### `ObjectStore.fromUrl(url: string, options?: Record<string, string>): ObjectStore`

Parses a store URL (e.g. `memory://`, `file:///path`, `s3://bucket/prefix`) and creates the corresponding object store.

### `parseUrl(url: string, options?: Record<string, string>): ParsedUrl`

Parses a store URL into a `ParsedUrl` object containing `store: ObjectStore` and `path: string`.

---

## Core Operations

### `put(path: string, bytes: Buffer): Promise<PutResult>`

Atomically writes data bytes to `path`.

### `putOpts(path: string, bytes: Buffer, options: PutOptionsParam): Promise<PutResult>`

Writes data bytes to `path` with specific options (e.g. `{ mode: 'create' }` or `{ mode: 'update', version: { eTag, version } }`).

### `get(path: string): Promise<GetResult>`

Fetches object at `path`. Returns `{ bytes: Buffer, meta: ObjectMeta }`.

### `getOpts(path: string, options: GetOptionsParam): Promise<GetResult>`

Fetches object at `path` with options (conditional fetch using `ifMatch`, `ifNoneMatch`, `ifModifiedSince`, `ifUnmodifiedSince`, or byte range `range`).

### `getRange(path: string, start: number, length: number): Promise<Buffer>`

Fetches a byte range `[start, start + length)` from object at `path`.

### `getRanges(path: string, ranges: RangeParam[]): Promise<Buffer[]>`

Performs vectored IO to fetch multiple byte ranges from object at `path`.

### `head(path: string): Promise<ObjectMeta>`

Fetches object metadata without downloading object bytes.

### `delete(path: string): Promise<void>`

Deletes object at `path`.

### `list(prefix?: string): Promise<ObjectMeta[]>`

Recursively lists objects under `prefix`.

### `listWithDelimiter(prefix?: string): Promise<ListResult>`

Lists objects and common prefixes (directories) under `prefix`. Returns `{ objects: ObjectMeta[], commonPrefixes: string[] }`.

### `copy(from: string, to: string): Promise<void>`

Copies object from `from` path to `to` path.

### `copyOpts(from: string, to: string, options: CopyOptionsParam): Promise<void>`

Copies object with options (e.g. `{ mode: 'create' }`).

### `rename(from: string, to: string): Promise<void>`

Renames/moves object from `from` path to `to` path.

### `renameOpts(from: string, to: string, options: RenameOptionsParam): Promise<void>`

Renames object with options.

---

## Interfaces

```ts
export interface ObjectMeta {
  location: string;
  lastModified: string;
  size: number;
  eTag?: string;
  version?: string;
}

export interface GetResult {
  bytes: Buffer;
  meta: ObjectMeta;
}

export interface PutResult {
  eTag?: string;
  version?: string;
}

export interface ListResult {
  objects: ObjectMeta[];
  commonPrefixes: string[];
}

export interface RangeParam {
  start: number;
  length: number;
}

export interface GetOptionsParam {
  ifMatch?: string;
  ifNoneMatch?: string;
  ifModifiedSince?: string;
  ifUnmodifiedSince?: string;
  range?: RangeParam;
  version?: string;
  head?: boolean;
}

export interface UpdateVersion {
  eTag?: string;
  version?: string;
}

export interface PutOptionsParam {
  mode?: "overwrite" | "create" | "update" | string;
  version?: UpdateVersion;
}

export interface CopyOptionsParam {
  mode?: "overwrite" | "create" | string;
}

export interface RenameOptionsParam {
  mode?: "overwrite" | "create" | string;
}
```
