import { spawnSync, spawn, ChildProcess } from "node:child_process";
import { writeFile } from "node:fs/promises";

const all = Promise.all.bind(Promise);

const SECOND = 1000;
const MINUTE = 60 * SECOND;

const OLD_BINARY_PATH = "/tmp/node-subtensor-old";
const NEW_BINARY_PATH = "/tmp/node-subtensor-new";
const CHAIN_SPEC_PATH = "/tmp/local.json";

const ONE_OPTIONS = {
  binaryPath: OLD_BINARY_PATH,
  basePath: "/tmp/one",
  name: "one",
  port: 30333,
  rpcPort: 9933,
  validator: true,
};

const TWO_OPTIONS = {
  binaryPath: OLD_BINARY_PATH,
  basePath: "/tmp/two",
  name: "two",
  port: 30334,
  rpcPort: 9944,
  validator: true,
};

const ALICE_OPTIONS = {
  binaryPath: OLD_BINARY_PATH,
  basePath: "/tmp/alice",
  name: "alice",
  port: 30335,
  rpcPort: 9955,
  validator: false,
};

type Node = { name: string; binaryPath: string; process: ChildProcess };

async function main() {
  await generateChainSpec();

  const one = startNode(ONE_OPTIONS);
  await started(one);

  const two = startNode(TWO_OPTIONS);
  await started(two);

  await all([peerCount(one, 1), peerCount(two, 1)]);
  await all([finalizedBlocks(one, 5), finalizedBlocks(two, 5)]);

  const alice = startNode(ALICE_OPTIONS);
  await started(alice);

  await all([peerCount(one, 2), peerCount(two, 2), peerCount(alice, 2)]);
  await all([finalizedBlocks(one, 10), finalizedBlocks(two, 10), finalizedBlocks(alice, 10)]);

  // Swap 'alice' node with the new binary
  await stop(alice);
  const aliceNew = startNode({ ...ALICE_OPTIONS, binaryPath: NEW_BINARY_PATH });
  await started(aliceNew);

  await all([peerCount(one, 2), peerCount(two, 2), peerCount(aliceNew, 2)]);
  await all([finalizedBlocks(one, 15), finalizedBlocks(two, 15), finalizedBlocks(aliceNew, 15)]);

  // Swap 'one' node with the new binary
  await stop(one);
  const oneNew = startNode({ ...ONE_OPTIONS, binaryPath: NEW_BINARY_PATH });
  await started(oneNew);

  await all([peerCount(two, 2), peerCount(aliceNew, 2), peerCount(oneNew, 2)]);
  await all([finalizedBlocks(oneNew, 20), finalizedBlocks(two, 20), finalizedBlocks(aliceNew, 20)]);

  // Swap 'two' node with the new binary
  await stop(two);
  const twoNew = startNode({ ...TWO_OPTIONS, binaryPath: NEW_BINARY_PATH });
  await started(twoNew);

  await all([peerCount(oneNew, 2), peerCount(twoNew, 2), peerCount(aliceNew, 2)]);
  await all([finalizedBlocks(oneNew, 50), finalizedBlocks(twoNew, 50), finalizedBlocks(aliceNew, 50)]);

  await all([stop(oneNew), stop(twoNew), stop(aliceNew)]);

  log("Test completed with success, binaries are compatible ✅");
}

// Generate the chain spec for the local network
const generateChainSpec = async () => {
  const result = spawnSync(
    OLD_BINARY_PATH,
    ["build-spec", "--disable-default-bootnode", "--raw", "--chain", "local"],
    { maxBuffer: 1024 * 1024 * 10 }, // 10MB
  );

  if (result.status !== 0) {
    throw new Error(`Failed to generate chain spec: ${result.stderr.toString()}`);
  }

  const stdout = result.stdout.toString();
  await writeFile(CHAIN_SPEC_PATH, stdout, { encoding: "utf-8" });
};

// Start a node with the given options
const startNode = (opts: {
  binaryPath: string;
  basePath: string;
  name: string;
  port: number;
  rpcPort: number;
  validator: boolean;
}): Node => {
  const process = spawn(opts.binaryPath, [
    `--${opts.name}`,
    ...["--chain", CHAIN_SPEC_PATH],
    ...["--base-path", opts.basePath],
    ...["--port", opts.port.toString()],
    ...["--rpc-port", opts.rpcPort.toString()],
    ...(opts.validator ? ["--validator"] : []),
    "--rpc-cors=all",
    "--allow-private-ipv4",
    "--discover-local",
    "--unsafe-force-node-key-generation",
  ]);

  process.on("error", (error) => console.error(`${opts.name} (error): ${error}`));
  process.on("close", (code) => log(`${opts.name}: process closed with code ${code}`));

  return { name: opts.name, binaryPath: opts.binaryPath, process };
};

const stop = (node: Node): Promise<void> => {
  return new Promise((resolve, reject) => {
    node.process.on("close", resolve);
    node.process.on("error", reject);

    if (!node.process.kill()) {
      reject(new Error(`Failed to stop ${node.name}`));
    }
  });
};

// Ensure the node has correctly started
const started = (node: Node, timeout = 30 * SECOND) => {
  const errorMessage = `Failed to start ${node.name} in time`;

  return innerEnsure(node, errorMessage, timeout, (data, ok) => {
    if (data.includes("💤 Idle")) {
      log(`${node.name}: started using ${node.binaryPath}`);
      ok();
    }
  });
};

// Ensure the node has reached the expected number of peers
const peerCount = (node: Node, expectedPeers: number, timeout = 30 * SECOND) => {
  const errorMessage = `Failed to reach ${expectedPeers} peers in time`;

  return innerEnsure(node, errorMessage, timeout, (data, ok) => {
    const maybePeers = /Idle \((?<peers>\d+) peers\)/.exec(data)?.groups?.peers;
    if (!maybePeers) return;

    const peers = parseInt(maybePeers);
    if (peers >= expectedPeers) {
      log(`${node.name}: reached ${expectedPeers} peers`);
      ok();
    }
  });
};

// Ensure the node has reached the expected number of finalized blocks
const finalizedBlocks = (node: Node, expectedFinalized: number, timeout = 10 * MINUTE) => {
  const errorMessage = `Failed to reach ${expectedFinalized} finalized blocks in time`;

  return innerEnsure(node, errorMessage, timeout, (data, ok) => {
    const maybeFinalized = /finalized #(?<blocks>\d+)/.exec(data)?.groups?.blocks;
    if (!maybeFinalized) return;

    const finalized = parseInt(maybeFinalized);
    if (finalized >= expectedFinalized) {
      log(`${node.name}: reached ${expectedFinalized} finalized blocks`);
      ok();
    }
  });
};

// Helper function to ensure a condition is met within a timeout
function innerEnsure(node: Node, errorMessage: string, timeout: number, f: (data: string, ok: () => void) => void) {
  return new Promise<void>((resolve, reject) => {
    const id = setTimeout(() => reject(new Error(errorMessage)), timeout);

    const fn = (data: string) =>
      f(data, () => {
        clearTimeout(id);
        node.process.stderr?.off("data", fn);
        resolve();
      });

    node.process.stderr?.on("data", fn);
  });
}

const log = (message: string) => console.log(`[${new Date().toISOString()}] ${message}`);

main();
