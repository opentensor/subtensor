#!/bin/bash
#
# Scaffold a new e2e test package.
#
# Usage:
#   ./bootstrap_package.sh <name>
#
# Example:
#   ./bootstrap_package.sh staking
#
set -e

if [ -z "$1" ]; then
  echo "Usage: $0 <package-name>"
  exit 1
fi

for cmd in jq yq; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: $cmd is required. Run ./setup_env.sh first."
    exit 1
  fi
done

NAME="$1"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIR="$SCRIPT_DIR/$NAME"
WORKSPACE="$SCRIPT_DIR/pnpm-workspace.yaml"

if [ -d "$DIR" ]; then
  echo "ERROR: Directory $DIR already exists"
  exit 1
fi

echo "==> Creating package e2e-$NAME..."
mkdir -p "$DIR/tests"

# -- package.json --
jq -n \
  --arg name "e2e-$NAME" \
  '{
    name: $name,
    version: "1.0.0",
    type: "module",
    scripts: { test: "vitest run" },
    dependencies: {
      "e2e-shared": "workspace:*",
      "@polkadot-api/descriptors": "file:../.papi/descriptors",
      "polkadot-api": "catalog:"
    },
    devDependencies: {
      "@types/node": "catalog:",
      "vitest": "catalog:"
    }
  }' > "$DIR/package.json"

# -- tsconfig.json --
jq -n '{
  compilerOptions: {
    target: "ES2022",
    module: "ESNext",
    moduleResolution: "bundler",
    esModuleInterop: true,
    strict: true,
    skipLibCheck: true,
    types: ["node", "vitest/globals"]
  }
}' > "$DIR/tsconfig.json"

# -- vitest.config.ts --
cat > "$DIR/vitest.config.ts" << 'EOF'
import { defineConfig } from "vitest/config";
import AlphabeticalSequencer from "e2e-shared/sequencer.js";

export default defineConfig({
  test: {
    globals: true,
    testTimeout: 120_000,
    hookTimeout: 300_000,
    fileParallelism: false,
    globalSetup: "./setup.ts",
    include: ["tests/**/*.test.ts"],
    sequence: {
      sequencer: AlphabeticalSequencer,
    },
  },
});
EOF

# -- setup.ts --
sed "s/__NAME__/$NAME/g" << 'SETUP_EOF' > "$DIR/setup.ts"
import { writeFile, readFile, rm, mkdir } from "node:fs/promises";
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

const CHAIN_SPEC_PATH = "/tmp/subtensor-e2e/__NAME__/chain-spec.json";
const STATE_FILE = "/tmp/subtensor-e2e/__NAME__/nodes.json";

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

// The local chain spec has 2 built-in authorities (One, Two).
// Add extra authorities here if needed.
const EXTRA_AUTHORITY_SEEDS: string[] = [];

type NodeConfig = Omit<NodeOptions, "binaryPath" | "chainSpec"> & {
  keySeed?: string;
};

// TODO: Adjust node configs for your test suite.
const NODE_CONFIGS: NodeConfig[] = [
  { name: "one", port: 30333, rpcPort: 9944, basePath: "/tmp/subtensor-e2e/__NAME__/one", validator: true },
  { name: "two", port: 30334, rpcPort: 9945, basePath: "/tmp/subtensor-e2e/__NAME__/two", validator: true },
];

export async function setup() {
  log(`Setting up ${NODE_CONFIGS.length}-node network for __NAME__ E2E tests`);
  log(`Binary path: ${BINARY_PATH}`);

  await mkdir("/tmp/subtensor-e2e/__NAME__", { recursive: true });

  await generateChainSpec(BINARY_PATH, CHAIN_SPEC_PATH, (spec) => {
    const patch = getGenesisPatch(spec);
    for (const seed of EXTRA_AUTHORITY_SEEDS) {
      addAuthority(patch, seed);
    }
  });

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
  log("Tearing down __NAME__ E2E test network");

  let state: NetworkState | undefined;
  try {
    const data = await readFile(STATE_FILE, "utf-8");
    state = JSON.parse(data);
  } catch {}

  for (const node of nodes) {
    try {
      await stop(node);
    } catch (e) {
      log(`Warning: failed to stop ${node.name}: ${e}`);
    }
  }

  if (state) {
    const ownPids = new Set(nodes.map((n) => n.process.pid));
    for (const nodeInfo of state.nodes) {
      if (!ownPids.has(nodeInfo.pid)) {
        try {
          process.kill(nodeInfo.pid, "SIGTERM");
          log(`Killed extra node ${nodeInfo.name} (pid ${nodeInfo.pid})`);
        } catch {}
      }
    }

  }

  await rm("/tmp/subtensor-e2e/__NAME__", { recursive: true, force: true });

  log("Teardown complete");
}
SETUP_EOF

# -- tests/00-basic.test.ts --
sed "s/__NAME__/$NAME/g" << 'TEST_EOF' > "$DIR/tests/00-basic.test.ts"
import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { readFile } from "node:fs/promises";
import type { PolkadotClient, TypedApi } from "polkadot-api";
import { subtensor } from "@polkadot-api/descriptors";
import type { NetworkState } from "../setup.js";
import {
  connectClient,
  createSigner,
  waitForFinalizedBlocks,
} from "e2e-shared/client.js";

let client: PolkadotClient;
let api: TypedApi<typeof subtensor>;
let state: NetworkState;

const alice = createSigner("//Alice");

beforeAll(async () => {
  const data = await readFile("/tmp/subtensor-e2e/__NAME__/nodes.json", "utf-8");
  state = JSON.parse(data);
  ({ client, api } = await connectClient(state.nodes[0].rpcPort));
  await waitForFinalizedBlocks(client, 3);
});

afterAll(() => {
  client?.destroy();
});

describe("__NAME__", () => {
  it("should produce finalized blocks", async () => {
    const block = await api.query.System.Number.getValue();
    expect(block).toBeGreaterThan(0);
  });
});
TEST_EOF

# -- Add to pnpm-workspace.yaml --
if ! yq '.packages[] | select(. == "'"$NAME"'")' "$WORKSPACE" | grep -q .; then
  yq -i '.packages += ["'"$NAME"'"]' "$WORKSPACE"
  echo "  Added '$NAME' to pnpm-workspace.yaml"
fi

echo "==> Created e2e/$NAME/"
echo ""
echo "Next steps:"
echo "  1. Edit $NAME/setup.ts to configure your network"
echo "  2. Add test-specific dependencies to $NAME/package.json"
echo "  3. Run: pnpm install"
echo "  4. Run: cd $NAME && pnpm test"
