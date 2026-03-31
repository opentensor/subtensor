import { writeFile, readFile, rm, mkdir } from "node:fs/promises";
import { generateChainSpec, insertKeys } from "e2e-shared/chainspec.js";
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

const BASE_DIR = "/tmp/subtensor-e2e/rate-limits";
const CHAIN_SPEC_PATH = `${BASE_DIR}/chain-spec.json`;
const STATE_FILE = `${BASE_DIR}/nodes.json`;

export type NetworkState = {
  binaryPath: string;
  chainSpec: string;
  nodes: {
    name: string;
    rpcPort: number;
    port: number;
    pid: number;
    basePath: string;
  }[];
};

const nodes: Node[] = [];
const BINARY_PATH = process.env.BINARY_PATH || "../../target/release/node-subtensor";

type NodeConfig = Omit<NodeOptions, "binaryPath" | "chainSpec"> & {
  keySeed?: string;
};

const NODE_CONFIGS: NodeConfig[] = [
  { name: "one", port: 30533, rpcPort: 9954, basePath: `${BASE_DIR}/one`, validator: true },
  { name: "two", port: 30534, rpcPort: 9955, basePath: `${BASE_DIR}/two`, validator: true },
  {
    name: "three",
    port: 30535,
    rpcPort: 9956,
    basePath: `${BASE_DIR}/three`,
    validator: true,
    keySeed: "//Three",
  },
];

export async function setup() {
  log(`Setting up ${NODE_CONFIGS.length}-node network for rate-limits E2E tests`);
  log(`Binary path: ${BINARY_PATH}`);

  await mkdir(BASE_DIR, { recursive: true });
  await generateChainSpec(BINARY_PATH, CHAIN_SPEC_PATH);

  for (const config of NODE_CONFIGS) {
    await rm(config.basePath, { recursive: true, force: true });
  }

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

  await Promise.all(nodes.map((node) => peerCount(node, nodes.length - 1)));
  log("All nodes peered");

  await Promise.all(nodes.map((node) => finalizedBlocks(node, 3)));
  log("All nodes finalized block 3");

  const state: NetworkState = {
    binaryPath: BINARY_PATH,
    chainSpec: CHAIN_SPEC_PATH,
    nodes: NODE_CONFIGS.map((config, index) => ({
      name: config.name,
      rpcPort: config.rpcPort,
      port: config.port,
      pid: nodes[index].process.pid!,
      basePath: config.basePath,
    })),
  };

  await writeFile(STATE_FILE, JSON.stringify(state, null, 2));
  log(`Network state written to ${STATE_FILE}`);
}

export async function teardown() {
  log("Tearing down rate-limits E2E test network");

  let state: NetworkState | undefined;
  try {
    state = JSON.parse(await readFile(STATE_FILE, "utf-8"));
  } catch {}

  for (const node of nodes) {
    try {
      await stop(node);
    } catch (error) {
      log(`Warning: failed to stop ${node.name}: ${error}`);
    }
  }

  if (state) {
    const ownPids = new Set(nodes.map((node) => node.process.pid));
    for (const nodeInfo of state.nodes) {
      if (!ownPids.has(nodeInfo.pid)) {
        try {
          process.kill(nodeInfo.pid, "SIGTERM");
        } catch {}
      }
    }
  }

  await rm(BASE_DIR, { recursive: true, force: true });
  log("Teardown complete");
}
