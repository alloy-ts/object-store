import { expect, test } from "vite-plus/test";
import fs from "node:fs";
import path from "node:path";

test("DTS - Verify generated declarations contain strongly typed options and methods", () => {
  const dtsPath = path.resolve(process.cwd(), "dist/index.d.ts");
  const content = fs.readFileSync(dtsPath, "utf-8");

  const expectedTypes = [
    "interface ObjectMeta",
    "interface PutResult",
    "interface GetResult",
    "interface ListResult",
    "interface Range",
    "interface GetRangeInput",
    "interface GetOptionsInput",
    "interface PutOptionsInput",
    "interface PutMultipartOptionsInput",
    "interface CopyOptionsInput",
    "interface RenameOptionsInput",
    "interface HeadOptionsInput",
    "interface DeleteOptionsInput",
    "interface ListOptionsInput",
    "interface PaginatedListOptionsInput",
    "interface PaginatedListResult",
    "getOpts(",
    "putOpts(",
    "putMultipartOpts(",
    "copyOpts(",
    "renameOpts(",
    "headOpts(",
    "deleteOpts(",
    "deleteStream(",
    "listOpts(",
    "listPaginated(",
  ];

  // `delete` takes no options and `deleteStream` replaces the old `deleteOpts`
  // alias, so neither an options interface nor an `Opts` overload may reappear.
  const removedTypes = ["interface DeleteOptionsInput", "deleteOpts("];

  for (const expected of expectedTypes) {
    expect(content).toContain(expected);
  }

  for (const removed of removedTypes) {
    expect(content).not.toContain(removed);
  }
});
