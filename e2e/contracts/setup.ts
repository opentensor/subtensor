import { rm, mkdir } from "node:fs/promises";
import {
  generateChainSpec,
  startNode,
  started,
  peerCount,
  finalizedBlocks,
  stop,
  nodeLog,
  destroyClient,
  getDevnetApi,
  sudoSetLockReductionInterval,
  log,
  type Node,
  type NodeOptions,
} from "e2e-shared";

const CHAIN_SPEC_PATH = "/tmp/subtensor-e2e/contracts/chain-spec.json";
const BASE_DIR = "/tmp/subtensor-e2e/contracts";

const BINARY_PATH = process.env.BINARY_PATH || "../../target/release/node-subtensor";

const nodes: Node[] = [];

// Use built-in validators "one" and "two" - they have auto-injected keys
type NodeConfig = Omit<NodeOptions, "binaryPath" | "chainSpec">;

const NODE_CONFIGS: NodeConfig[] = [
  { name: "one", port: 30533, rpcPort: 9944, basePath: `${BASE_DIR}/one`, validator: true },
  { name: "two", port: 30534, rpcPort: 9945, basePath: `${BASE_DIR}/two`, validator: true },
];

async function startNetwork() {
  nodeLog(`Setting up ${NODE_CONFIGS.length}-node network for contracts E2E tests`);
  nodeLog(`Binary path: ${BINARY_PATH}`);

  await mkdir(BASE_DIR, { recursive: true });

  // Generate local chain spec (built-in has One and Two as authorities)
  await generateChainSpec(BINARY_PATH, CHAIN_SPEC_PATH);

  // Clean up old base paths
  for (const config of NODE_CONFIGS) {
    await rm(config.basePath, { recursive: true, force: true });
  }

  // Start all validator nodes
  for (const config of NODE_CONFIGS) {
    const node = startNode({
      binaryPath: BINARY_PATH,
      chainSpec: CHAIN_SPEC_PATH,
      ...config,
    });
    nodes.push(node);
    await started(node);
  }

  const all = Promise.all.bind(Promise);

  // Wait for nodes to peer with each other
  await all(nodes.map((n) => peerCount(n, nodes.length - 1)));
  nodeLog("All nodes peered");

  // Wait for block finalization
  await all(nodes.map((n) => finalizedBlocks(n, 3)));
  nodeLog("All nodes finalized block 3");
}

async function stopNetwork() {
  nodeLog("Stopping contracts network");

  for (const node of nodes) {
    try {
      await stop(node);
    } catch (e) {
      nodeLog(`Warning: failed to stop ${node.name}: ${e}`);
    }
  }

  // Clean up the suite directory
  await rm(BASE_DIR, { recursive: true, force: true });

  nodeLog("Network stopped");
}

before(async function () {
  // Increase timeout for network startup (2 minutes)
  this.timeout(120000);

  // Start the network
  await startNetwork();

  // Connect to the network and configure for tests
  const api = await getDevnetApi();
  log.info("Setup: set lock reduction interval to 1 for instant lock cost decay");

  // Set lock reduction interval to 1 block to make network registration lock cost decay instantly.
  await sudoSetLockReductionInterval(api, 1);
});

after(async function () {
  // Increase timeout for cleanup
  this.timeout(30000);

  // Destroy the API client first
  destroyClient();

  // Stop the network
  await stopNetwork();
});
