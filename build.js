import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const require = createRequire(import.meta.url);
const { NapiCli } = require("@napi-rs/cli");

async function run() {
  const args = process.argv.slice(2);
  const isRelease = args.includes("--release") || args.includes("-r");
  const useNapiCross = args.includes("--use-napi-cross");
  const crossCompile = args.includes("--cross-compile") || args.includes("-x");
  const useCross = args.includes("--use-cross");

  const targetIdx = args.findIndex((a) => a === "--target" || a === "-t");
  const target = targetIdx !== -1 && args[targetIdx + 1] ? args[targetIdx + 1] : undefined;

  const cli = new NapiCli();
  await cli.build({
    platform: true,
    esm: true,
    outputDir: "./dist",
    release: isRelease,
    target,
    useNapiCross,
    crossCompile,
    useCross,
  });

  const nodeBinary = "./dist/obj-store.linux-x64-gnu.node";
  if (!fs.existsSync(nodeBinary)) {
    const profile = isRelease ? "release" : "debug";
    const possiblePaths = [
      `./target/x86_64-unknown-linux-gnu/${profile}/libmy_addon_native.so`,
      `./target/${profile}/libmy_addon_native.so`,
    ];
    for (const p of possiblePaths) {
      if (fs.existsSync(p)) {
        fs.copyFileSync(p, nodeBinary);
        break;
      }
    }
  }

  if (fs.existsSync("./dist")) {
    for (const file of fs.readdirSync("./dist")) {
      fs.copyFileSync(path.join("./dist", file), path.join(".", file));
    }
  }
}

void run().catch((err) => {
  console.error(err);
  process.exit(1);
});
