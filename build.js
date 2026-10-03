import { NapiCli, createBuildCommand } from "@napi-rs/cli";

async function run() {
  const buildCommand = createBuildCommand(process.argv.slice(2));
  const cli = new NapiCli();

  const options = buildCommand.getOptions();
  const buildOptions = {
    ...options,
    platform: options.platform ?? true,
    esm: options.esm ?? true,
    cargoOptions: buildCommand.cargoOptions,
  };

  const { task } = await cli.build(buildOptions);
  await task;
}

void run().catch((err) => {
  console.error(err);
  process.exit(1);
});
