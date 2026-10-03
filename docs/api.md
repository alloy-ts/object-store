# @lib/object-store API Reference

High-performance Node.js native bindings for the Apache Arrow [`object_store`](https://docs.rs/object_store/latest/object_store/) crate using NAPI-RS.

---

## Table of Contents

- [ObjectStore Class](#objectstore-class)
  - [Constructors & Factory Methods](#constructors--factory-methods)
  - [Methods](#methods)
    - [`put(path, payload)`](#putpath-payload)
    - [`putOpts(path, payload, options)`](#putoptspath-payload-options)
    - [`get(path)`](#getpath)
    - [`getResult(path)`](#getresultpath)
    - [`getOpts(path, options)`](#getoptspath-options)
    - [`getRanges(path, ranges)`](#getrangespath-ranges)
    - [`head(path)`](#headpath)
    - [`delete(path)`](#deletepath)
    - [`deleteOpts(path, options)`](#deleteoptspath-options)
    - [`list(prefix?)`](#listprefix)
    - [`listWithOffset(prefix, offset)`](#listwithoffsetprefix-offset)
    - [`copy(from, to)`](#copyfrom-to)
    - [`copyOpts(from, to, options)`](#copyoptsfrom-to-options)
    - [`rename(from, to)`](#renamefrom-to)
    - [`renameOpts(from, to, options)`](#renameoptsfrom-to-options)
- [Feature Flags](#feature-flags)
  - [`getFeatures()`](#getfeatures)
- [Types & Interfaces](#types--interfaces)

---

## ObjectStore Class

The `ObjectStore` class provides a unified, asynchronous interface for interacting with local files and cloud object storage services (S3, GCS, Azure, Memory, WebDAV, etc.).

### Constructors & Factory Methods

#### `new ObjectStore(url?: string, options?: Record<string, string>)`

Constructs an `ObjectStore` instance. If `url` is omitted or empty, defaults to an in-memory store (`memory://`).

#### `ObjectStore.memory(): ObjectStore`

Creates an in-memory object store.

#### `ObjectStore.local(rootPath: string): ObjectStore`

Creates a local filesystem object store rooted at `rootPath`.

#### `ObjectStore.parseUrl(url: string, options?: Record<string, string>): ParseUrlResult`

Parses a store URL (e.g. `memory://`, `file:///path`, `s3://bucket/path`) and returns `{ store: ObjectStore, path: string }`.

#### `parseUrl(url: string, options?: Record<string, string>): ParseUrlResult`

Standalone helper function equivalent to `ObjectStore.parseUrl`.

---

### Methods

#### `put(path: string, payload: Uint8Array | Buffer): Promise<PutResult>`

Atomically uploads object bytes to `path`.

#### `putOpts(path: string, payload: Uint8Array | Buffer, options?: PutOptions): Promise<PutResult>`

Uploads object bytes to `path` with options such as mode (`'overwrite' | 'create' | 'update'`) and key-value tags.

#### `get(path: string): Promise<Buffer>`

Fetches all object bytes from `path`.

#### `getResult(path: string): Promise<GetResult>`

Fetches the object at `path` along with its metadata (`{ meta, bytes }`).

#### `getOpts(path: string, options?: GetOptions): Promise<Buffer>`

Fetches object bytes with options such as byte ranges or HTTP preconditions (`ifMatch`, `ifNoneMatch`, `ifModifiedSince`, `ifUnmodifiedSince`).

#### `getRanges(path: string, ranges: RangeInput[]): Promise<Buffer[]>`

Performs vectored IO, fetching non-contiguous byte ranges in parallel.

#### `head(path: string): Promise<ObjectMeta>`

Fetches object metadata (`location`, `size`, `lastModified`, `eTag`, `version`) without downloading the payload.

#### `delete(path: string): Promise<void>`

Deletes the object at `path`.

#### `deleteOpts(path: string, options?: DeleteOptions): Promise<void>`

Deletes the object at `path` with options.

#### `list(prefix?: string): Promise<ObjectMeta[]>`

Lists object metadata recursively under `prefix`.

#### `listWithOffset(prefix: string | null, offset: string): Promise<ObjectMeta[]>`

Lists object metadata starting after `offset`.

#### `copy(from: string, to: string): Promise<void>`

Copies an object from `from` to `to`.

#### `copyOpts(from: string, to: string, options?: CopyOptions): Promise<void>`

Copies an object with options.

#### `rename(from: string, to: string): Promise<void>`

Renames / moves an object from `from` to `to`.

#### `renameOpts(from: string, to: string, options?: RenameOptions): Promise<void>`

Renames / moves an object with options.

---

## Feature Flags

### `getFeatures(): FeatureFlags`

Queries the active build-time feature flags in the native Rust library.

```ts
import { getFeatures } from "@lib/object-store";

console.log(getFeatures());
// { fs: true, tokio: true, aws: false, azure: false, gcp: false, http: false }
```

---

## Types & Interfaces

```ts
export interface ObjectMeta {
  location: string;
  lastModified: string;
  size: number;
  eTag?: string;
  version?: string;
}

export interface PutResult {
  eTag?: string;
  version?: string;
}

export interface PutOptions {
  mode?: "overwrite" | "create" | "update";
  eTag?: string;
  version?: string;
  tags?: Record<string, string>;
}

export interface GetOptions {
  range?: { start?: number; end?: number };
  rangeStart?: number;
  rangeEnd?: number;
  ifMatch?: string;
  ifNoneMatch?: string;
  ifModifiedSince?: string;
  ifUnmodifiedSince?: string;
  head?: boolean;
}

export interface GetResult {
  meta: ObjectMeta;
  bytes: Buffer;
}

export interface RangeInput {
  start: number;
  end: number;
}

export interface CopyOptions {
  mode?: "overwrite" | "create";
}

export interface RenameOptions {
  targetMode?: "overwrite" | "create";
}

export interface DeleteOptions {}

export interface FeatureFlags {
  fs: boolean;
  tokio: boolean;
  aws: boolean;
  azure: boolean;
  gcp: boolean;
  http: boolean;
}
```
