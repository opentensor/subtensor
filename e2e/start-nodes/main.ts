import {
  generateChainSpec,
  startNode,
  detach,
  started,
  peerCount,
  finalizedBlocks,
  log,
  all,
  NodeOptions,
} from "./lib.js";

const OLD_BINARY_PATH = "../../target/release/node-subtensor";
const CHAIN_SPEC_PATH = "/tmp/local.json";

const ONE_OPTIONS: NodeOptions = {
  binaryPath: OLD_BINARY_PATH,
  basePath: "/tmp/one",
  name: "one",
  port: 30333,
  rpcPort: 9933,
  validator: true,
};

const TWO_OPTIONS: NodeOptions = {
  binaryPath: OLD_BINARY_PATH,
  basePath: "/tmp/two",
  name: "two",
  port: 30334,
  rpcPort: 9944,
  validator: true,
};

async function main() {
  await generateChainSpec({ binaryPath: OLD_BINARY_PATH, outputPath: CHAIN_SPEC_PATH });

  const one = startNode({ ...ONE_OPTIONS, chainSpecPath: CHAIN_SPEC_PATH });
  await started(one);

  const two = startNode({ ...TWO_OPTIONS, chainSpecPath: CHAIN_SPEC_PATH });
  await started(two);

  await all([peerCount(one, 1), peerCount(two, 1)]);
  await all([finalizedBlocks(one, 5), finalizedBlocks(two, 5)]);

  log("Validators started ✅");

  detach(one);
  detach(two);
}

main();
