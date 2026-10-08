# API Reference - `@alloy-ts/object-store`

`@alloy-ts/object-store` provides high-performance Node.js native bindings for Apache Arrow's `object_store` crate.

---

## Table of Contents

- [ObjectStore Class](#objectstore-class)
  - [Constructors](#constructors)
  - [Operations](#operations)
    - [`put(path, data, options?)`](#putpath-data-options)
    - [`putOpts(path, data, options)`](#putoptspath-data-options)
    - [`get(path, options?)`](#getpath-options)
    - [`getOpts(path, options)`](#getoptspath-options)
    - [`getWithMeta(path)`](#getwithmetapath)
    - [`head(path, options?)`](#headpath-options)
    - [`headOpts(path, options?)`](#headoptspath-options)
    - [`delete(path, options?)`](#deletepath-options)
    - [`deleteOpts(path, options?)`](#deleteoptspath-options)
    - [`list(prefix?, options?)`](#listprefix-options)
    - [`listOpts(prefix?, options?)`](#listoptsprefix-options)
    - [`listWithDelimiter(prefix?)`](#listwithdelimiterprefix)
    - [`copy(from, to, options?)`](#copyfrom-to-options)
    - [`copyOpts(from, to, options?)`](#copyoptsfrom-to-options)
    - [`rename(from, to, options?)`](#renamefrom-to-options)
    - [`renameOpts(from, to, options?)`](#renameoptsfrom-to-options)
    - [`getRanges(path, ranges)`](#getrangespath-ranges)
- [Types & Interfaces](#types--interfaces)

---

## ObjectStore Class

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

#### `list(prefix?: string, options?: ListOptionsInput): Promise<Array<ObjectMeta>>`

Recursively lists objects matching `prefix`.

#### `listOpts(prefix?: string, options?: ListOptionsInput): Promise<Array<ObjectMeta>>`

Alias for `list`.

#### `listWithDelimiter(prefix?: string): Promise<ListResult>`

Lists objects and common prefixes (directories) at `prefix`.

#### `copy(from: string, to: string, options?: CopyOptionsInput): Promise<void>`

Copies object from path `from` to `to`.

#### `copyOpts(from: string, to: string, options?: CopyOptionsInput): Promise<void>`

Alias for `copy`.

#### `rename(from: string, to: string, options?: RenameOptionsInput): Promise<void>`

Renames/moves object from path `from` to `to`.

#### `renameOpts(from: string, to: string, options?: RenameOptionsInput): Promise<void>`

Alias for `rename`.

#### `getRanges(path: string, ranges: Array<Range>): Promise<Array<Buffer>>`

Performs vectored IO, fetching non-contiguous byte ranges in parallel.

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
