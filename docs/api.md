# API Reference - `@alloy-ts/object-store`

`@alloy-ts/object-store` provides high-performance Node.js native bindings for Apache Arrow's `object_store` crate.

---

## Table of Contents

- [ObjectStore Class](#objectstore-class)
  - [Constructors](#constructors)
  - [Operations](#operations)
    - [`put(path, data, options?)`](#putpath-data-options)
    - [`putOpts(path, data, options)`](#putoptspath-data-options)
    - [`putMultipart(path)`](#putmultipartpath)
    - [`get(path, options?)`](#getpath-options)
    - [`getOpts(path, options)`](#getoptspath-options)
    - [`getWithMeta(path)`](#getwithmetapath)
    - [`getRange(path, range)`](#getrangepath-range)
    - [`getRanges(path, ranges)`](#getrangespath-ranges)
    - [`head(path, options?)`](#headpath-options)
    - [`headOpts(path, options?)`](#headoptspath-options)
    - [`delete(path, options?)`](#deletepath-options)
    - [`deleteOpts(path, options?)`](#deleteoptspath-options)
    - [`deleteStream(locations)`](#deletestreamlocations)
    - [`list(prefix?, options?)`](#listprefix-options)
    - [`listOpts(prefix?, options?)`](#listoptsprefix-options)
    - [`listWithDelimiter(prefix?)`](#listwithdelimiterprefix)
    - [`listWithOffset(prefix, offset)`](#listwithoffsetprefix-offset)
    - [`copy(from, to, options?)`](#copyfrom-to-options)
    - [`copyOpts(from, to, options?)`](#copyoptsfrom-to-options)
    - [`copyIfNotExists(from, to)`](#copyifnotexistsfrom-to)
    - [`rename(from, to, options?)`](#renamefrom-to-options)
    - [`renameOpts(from, to, options?)`](#renameoptsfrom-to-options)
    - [`renameIfNotExists(from, to)`](#renameifnotexistsfrom-to)
- [Store Classes](#store-classes)
- [Types & Interfaces](#types--interfaces)

---

## ObjectStore Class

`ObjectStore` is the **core API** of this package. Its operation surface mirrors Rust's
`ObjectStoreExt` extension trait (the convenience API over the `object_store` crate's
`ObjectStore` trait), so the same calls work uniformly across the in-memory store, the
local filesystem store, and any wrapped or decorated store (limit, throttle, chunked, ...).

Concrete store classes do not re-implement operations; they expose their
store-specific configuration and hand back a core `ObjectStore` view of the same
backing storage via `asObjectStore()` (see [Store Classes](#store-classes)).

### Constructors

#### `ObjectStore.createInMemory(): ObjectStore`

Creates an in-memory object store.

#### `ObjectStore.createLocal(rootPath: string): ObjectStore`

Creates a local filesystem object store rooted at `rootPath`.

#### `ObjectStore.parseUrl(url: string, options?: Record<string, string>): ObjectStore`

Parses a store URL with optional configuration key-value options. Only `file://` and `memory://` schemes are supported (cloud/reqwest features are disabled); `s3://`, `az://`, `gs://`, and `http(s)://` URLs return an error.

---

### Operations

#### `put(path: string, data: Buffer, options?: PutOptionsInput): Promise<PutResult>`

Atomically writes `data` to `path`.

#### `putOpts(path: string, data: Buffer, options: PutOptionsInput): Promise<PutResult>`

Alias for `put` with explicit options.

#### `putMultipart(path: string): Promise<MultipartUpload>`

Starts a multipart upload and returns a handle (`putPart` / `complete` / `abort`).
Prefer `put` for small payloads; use this for large or streaming uploads.

#### `get(path: string, options?: GetOptionsInput): Promise<Buffer>`

Fetches object byte content.

#### `getOpts(path: string, options: GetOptionsInput): Promise<Buffer>`

Fetches object byte content with conditional or range options.

#### `getWithMeta(path: string): Promise<GetResult>`

Fetches object byte content along with metadata.

#### `head(path: string, options?: HeadOptionsInput): Promise<ObjectMeta>`

Fetches object metadata without downloading content.

#### `headOpts(path: string, options?: HeadOptionsInput): Promise<ObjectMeta>`

Alias for `head`.

#### `delete(path: string, options?: DeleteOptionsInput): Promise<void>`

Deletes an object.

#### `deleteOpts(path: string, options?: DeleteOptionsInput): Promise<void>`

Alias for `delete`.

#### `deleteStream(locations: Array<string>): Promise<Array<DeleteStreamResult>>`

Bulk-deletes all objects at `locations`. Backends with native bulk delete
(S3, Azure) batch requests; others delete concurrently. Returns one result per
input location, in order:

```typescript
export interface DeleteStreamResult {
  path: string;
  error?: string;
}
```

Note: whether deleting a non-existent object errors or succeeds depends on the
backend (S3 and in-memory return success; local, GCP, and Azure error).

#### `list(prefix?: string, options?: ListOptionsInput): Promise<Array<ObjectMeta>>`

Recursively lists objects matching `prefix`.

#### `listOpts(prefix?: string, options?: ListOptionsInput): Promise<Array<ObjectMeta>>`

Alias for `list`.

#### `listWithDelimiter(prefix?: string): Promise<ListResult>`

Lists objects with the given prefix and an implementation-specific delimiter.
Non-recursive: returns common prefixes ("directories") in addition to object
metadata.

#### `listWithOffset(prefix?: string, offset: string): Promise<Array<ObjectMeta>>`

Lists all objects with the given prefix whose location is greater than
`offset` (exclusive). Some stores (S3, GCS) can push the offset down to reduce
network requests.

#### `copy(from: string, to: string, options?: CopyOptionsInput): Promise<void>`

Copies object from path `from` to `to`.

#### `copyOpts(from: string, to: string, options?: CopyOptionsInput): Promise<void>`

Alias for `copy`.

#### `copyIfNotExists(from: string, to: string): Promise<void>`

Copies object from path `from` to `to`, only if the destination is empty.
Errors if an object already exists at `to`. Atomic when the backend supports it.

#### `rename(from: string, to: string, options?: RenameOptionsInput): Promise<void>`

Renames/moves object from path `from` to `to`. By default this is a copy of
the source followed by a delete of the source.

#### `renameOpts(from: string, to: string, options?: RenameOptionsInput): Promise<void>`

Alias for `rename`.

#### `renameIfNotExists(from: string, to: string): Promise<void>`

Moves object from path `from` to `to`, only if the destination does not already
exist. Errors if an object already exists at `to`.

#### `getRange(path: string, range: Range): Promise<Buffer>`

Fetches the bytes stored at `path` within the half-open byte range
`[range.start, range.end)`.

#### `getRanges(path: string, ranges: Array<Range>): Promise<Array<Buffer>>`

Performs vectored IO, fetching non-contiguous byte ranges in parallel.

---

## Store Classes

Concrete store classes hold a specific backend type so they can expose
store-specific configuration. They do **not** re-implement object operations;
call `asObjectStore()` to get the core `ObjectStore` API backed by the same
storage.

### `InMemory`

- `new InMemory()` — create new in-memory storage.
- `fork(): InMemory` — snapshot the current contents into a brand-new store.
- `asObjectStore(): ObjectStore` — core operation surface over this store.
- Also accepted by `MultipartStore.fromInMemory(store)` and `ThrottledStore.new(store, config?)`.

```typescript
const store = new InMemory();
const view = store.asObjectStore();
await view.put("hello.txt", Buffer.from("world"));
```

### `LocalFileSystem`

- `new LocalFileSystem()` — storage rooted at the filesystem root.
- `LocalFileSystem.newWithPrefix(prefix: string)` — storage rooted at `prefix`.
- Builder methods (`withFsync`, `withAutomaticCleanup`, ...) return a new store.
- `pathToFilesystem(location: string): string` — absolute filesystem path of an object.
- `asObjectStore(): ObjectStore` — core operation surface over this store.

### `LimitStore`

- `LimitStore.new(store: ObjectStore, maxRequests: number)` — bounds concurrent
  outstanding operations.
- `asObjectStore(): ObjectStore` — core operation surface (concurrency-limited).

### `ThrottledStore`

- `ThrottledStore.new(inner: InMemory, config?: ThrottleConfig)` — wraps an
  in-memory store with deterministic per-call sleeps.
- `asObjectStore(): ObjectStore` / `asMultipartStore(): MultipartStore`.
- `getConfig(): ThrottleConfig` / `configure(config: ThrottleConfig)`.

### `ChunkedStore`

- `ChunkedStore.new(store: ObjectStore, chunkSize: number)` — forces `get`
  responses to be returned in fixed-size chunks (test helper).
- `asObjectStore(): ObjectStore` — core operation surface (chunked gets).

---

## Types & Interfaces

```typescript
export interface ObjectMeta {
  location: string;
  lastModified: number;
  size: number;
  eTag?: string;
  version?: string;
}

export interface PutResult {
  eTag?: string;
  version?: string;
}

export interface GetResult {
  bytes: Buffer;
  meta: ObjectMeta;
}

export interface ListResult {
  objects: Array<ObjectMeta>;
  commonPrefixes: Array<string>;
}

export interface Range {
  start: number;
  end: number;
}

export interface GetRangeInput {
  start?: number;
  end?: number;
  offset?: number;
  suffix?: number;
}

export interface GetOptionsInput {
  ifMatch?: string;
  ifNoneMatch?: string;
  ifModifiedSince?: number;
  ifUnmodifiedSince?: number;
  range?: GetRangeInput;
  version?: string;
  head?: boolean;
}

export interface UpdateVersionInput {
  eTag?: string;
  version?: string;
}

export interface PutOptionsInput {
  modeOverwrite?: boolean;
  modeCreate?: boolean;
  modeUpdate?: UpdateVersionInput;
}

export interface CopyOptionsInput {
  ifNotExists?: boolean;
}

export interface RenameOptionsInput {
  targetModeOverwrite?: boolean;
  targetModeCreate?: boolean;
}

export interface HeadOptionsInput {
  ifMatch?: string;
  ifNoneMatch?: string;
  ifModifiedSince?: number;
  ifUnmodifiedSince?: number;
  version?: string;
}

export interface DeleteOptionsInput {
  dummy?: boolean;
}

export interface ListOptionsInput {
  offset?: string;
}
```
