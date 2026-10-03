import { NapiCli, createBuildCommand } from "@napi-rs/cli";

async function run() {
  const args = process.argv.slice(2);
  const buildCommand = createBuildCommand(args);
  const cli = new NapiCli();
  const options = buildCommand.getOptions();

  let cargoOptions = buildCommand.cargoOptions ?? [];

  // Check if --features option was passed in CLI args
  const featureIdx = args.findIndex((a) => a === "--features" || a === "-F");
  if (featureIdx !== -1 && args[featureIdx + 1] && !cargoOptions.includes("--features")) {
    cargoOptions = ["--features", args[featureIdx + 1], ...cargoOptions];
  }

  const buildOptions = {
    ...options,
    platform: options.platform ?? true,
    esm: options.esm ?? true,
    cargoOptions,
    outputDir: options.outputDir ?? "build",
  };

  await cli.build(buildOptions);
}

void run().catch((err) => {
  console.error(err);
  process.exit(1);
});
