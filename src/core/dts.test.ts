import { expect, test } from "vite-plus/test";
import fs from "node:fs";
import path from "node:path";

test("DTS - Verify generated declarations contain strongly typed options and methods", () => {
  const dtsPath = path.resolve(process.cwd(), "index.d.ts");
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
    "interface CopyOptionsInput",
    "interface RenameOptionsInput",
    "interface HeadOptionsInput",
    "interface DeleteOptionsInput",
    "interface ListOptionsInput",
    "getOpts(",
    "putOpts(",
    "copyOpts(",
    "renameOpts(",
    "headOpts(",
    "deleteOpts(",
    "listOpts(",
  ];

  for (const expected of expectedTypes) {
    expect(content).toContain(expected);
  }
});
