import { writeFile, readFile, rm } from "node:fs/promises";
import {
  generateChainSpec,
  insertKeys,
  getGenesisPatch,
  addAuthority,
} from "e2e-shared/chainspec.js";
import {
  startNode,
  started,
  peerCount,
  finalizedBlocks,
  stop,
  log,
  type Node,
  type NodeOptions,
} from "e2e-shared/node.js";

const CHAIN_SPEC_PATH = "/tmp/e2e-shield-chain-spec.json";
const STATE_FILE = "/tmp/e2e-shield-nodes.json";

export type NetworkState = {
  binaryPath: string;
  chainSpec: string;
  nodes: { name: string; rpcPort: number; port: number; pid: number; basePath: string }[];
};

const nodes: Node[] = [];

const BINARY_PATH = process.env.BINARY_PATH || "../../target/debug/node-subtensor";

// The local chain spec has 2 built-in authorities (One, Two).
// We add "Three" dynamically by patching the chain spec JSON.
const EXTRA_AUTHORITY_SEEDS = ["Three"];

type NodeConfig = Omit<NodeOptions, "binaryPath" | "chainSpec"> & {
  keySeed?: string;
};

const NODE_CONFIGS: NodeConfig[] = [
  { name: "one", port: 30333, rpcPort: 9944, basePath: "/tmp/e2e-shield-one", validator: true },
  { name: "two", port: 30334, rpcPort: 9945, basePath: "/tmp/e2e-shield-two", validator: true },
  {
    name: "three",
    port: 30335,
    rpcPort: 9946,
    basePath: "/tmp/e2e-shield-three",
    validator: true,
    keySeed: "//Three",
  },
];

export async function setup() {
  log(`Setting up ${NODE_CONFIGS.length}-node network for shield E2E tests`);
  log(`Binary path: ${BINARY_PATH}`);

  await generateChainSpec(BINARY_PATH, CHAIN_SPEC_PATH, (spec) => {
    const patch = getGenesisPatch(spec);
    for (const seed of EXTRA_AUTHORITY_SEEDS) {
      addAuthority(patch, seed);
    }
  });

  for (const config of NODE_CONFIGS) {
    await rm(config.basePath, { recursive: true, force: true });
  }

  // Insert keys for authority nodes that don't have built-in substrate shortcuts.
  for (const config of NODE_CONFIGS) {
    if (config.keySeed) {
      insertKeys(BINARY_PATH, config.basePath, CHAIN_SPEC_PATH, config.keySeed);
    }
  }

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

  await all(nodes.map((n) => peerCount(n, nodes.length - 1)));
  log("All nodes peered");

  await all(nodes.map((n) => finalizedBlocks(n, 3)));
  log("All nodes finalized block 3");

  const state: NetworkState = {
    binaryPath: BINARY_PATH,
    chainSpec: CHAIN_SPEC_PATH,
    nodes: NODE_CONFIGS.map((c, i) => ({
      name: c.name,
      rpcPort: c.rpcPort,
      port: c.port,
      pid: nodes[i].process.pid!,
      basePath: c.basePath,
    })),
  };

  await writeFile(STATE_FILE, JSON.stringify(state, null, 2));
  log("Network state written to " + STATE_FILE);
}

export async function teardown() {
  log("Tearing down shield E2E test network");

  // Read the state file to find ALL nodes (including extras added by scaling tests).
  let state: NetworkState | undefined;
  try {
    const data = await readFile(STATE_FILE, "utf-8");
    state = JSON.parse(data);
  } catch {}

  // Stop nodes we have handles to (from globalSetup).
  for (const node of nodes) {
    try {
      await stop(node);
    } catch (e) {
      log(`Warning: failed to stop ${node.name}: ${e}`);
    }
  }

  // Kill any extra nodes (added by scaling tests) by PID.
  if (state) {
    const ownPids = new Set(nodes.map((n) => n.process.pid));
    for (const nodeInfo of state.nodes) {
      if (!ownPids.has(nodeInfo.pid)) {
        try {
          process.kill(nodeInfo.pid, "SIGTERM");
          log(`Killed extra node ${nodeInfo.name} (pid ${nodeInfo.pid})`);
        } catch {
          // Already dead, ignore.
        }
      }
    }

    // Clean up all base paths.
    for (const nodeInfo of state.nodes) {
      await rm(nodeInfo.basePath, { recursive: true, force: true });
    }
  }

  try {
    await rm(STATE_FILE, { force: true });
    await rm(CHAIN_SPEC_PATH, { force: true });
  } catch {}

  log("Teardown complete");
}

export async function readNetworkState(): Promise<NetworkState> {
  const data = await readFile(STATE_FILE, "utf-8");
  return JSON.parse(data);
}
