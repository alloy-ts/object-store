import { createBuildCommand, NapiCli } from "@napi-rs/cli";
import fs from "node:fs";
import path from "node:path";

const buildCommand = createBuildCommand(process.argv.slice(2));
const options = buildCommand.getOptions();

const outputDirWasDefault = !options.outputDir || options.outputDir === ".";
if (outputDirWasDefault) {
  options.outputDir = ".staging";
}

const cli = new NapiCli();

const buildOptions = {
  ...options,
  cargoOptions: buildCommand.cargoOptions,
};

const { task } = await cli.build(buildOptions);

await task;

if (outputDirWasDefault && fs.existsSync(".staging")) {
  for (const file of fs.readdirSync(".staging")) {
    fs.copyFileSync(path.join(".staging", file), path.join(".", file));
  }
  fs.rmSync(".staging", { recursive: true, force: true });
}
