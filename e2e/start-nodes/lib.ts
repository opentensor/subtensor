import { spawnSync, spawn, ChildProcess } from "node:child_process";
import { writeFile } from "node:fs/promises";

export const SECOND = 1000;
export const MINUTE = 60 * SECOND;

export type Node = { name: string; binaryPath: string; process: ChildProcess };

export type NodeOptions = {
  binaryPath: string;
  basePath: string;
  name: string;
  port: number;
  rpcPort: number;
  validator: boolean;
};

export type ChainSpecOptions = {
  binaryPath: string;
  outputPath: string;
  chain?: string;
};

// Generate the chain spec for the local network
export const generateChainSpec = async (opts: ChainSpecOptions): Promise<void> => {
  const result = spawnSync(
    opts.binaryPath,
    ["build-spec", "--disable-default-bootnode", "--raw", "--chain", opts.chain ?? "local"],
    { maxBuffer: 1024 * 1024 * 10 }, // 10MB
  );

  if (result.error) {
    throw new Error(`Failed to spawn process: ${result.error.message}`);
  }

  if (result.status !== 0) {
    throw new Error(`Failed to generate chain spec: ${result.stderr?.toString() ?? "unknown error"}`);
  }

  const stdout = result.stdout.toString();
  await writeFile(opts.outputPath, stdout, { encoding: "utf-8" });
};

// Start a node with the given options
export const startNode = (opts: NodeOptions & { chainSpecPath: string }): Node => {
  const process = spawn(opts.binaryPath, [
    `--${opts.name}`,
    ...["--chain", opts.chainSpecPath],
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

// Detach the node process so Node.js can exit while it keeps running
export const detach = (node: Node): void => {
  node.process.unref();
  node.process.stdout?.unref();
  node.process.stderr?.unref();
  node.process.stdin?.unref();
};

// Ensure the node has correctly started
export const started = (node: Node, timeout = 30 * SECOND): Promise<void> => {
  const errorMessage = `Failed to start ${node.name} in time`;

  return innerEnsure(node, errorMessage, timeout, (data, ok) => {
    if (data.includes("💤 Idle")) {
      log(`${node.name}: started using ${node.binaryPath}`);
      ok();
    }
  });
};

// Ensure the node has reached the expected number of peers
export const peerCount = (node: Node, expectedPeers: number, timeout = 30 * SECOND): Promise<void> => {
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
export const finalizedBlocks = (node: Node, expectedFinalized: number, timeout = 10 * MINUTE): Promise<void> => {
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

export const log = (message: string) => console.log(`[${new Date().toISOString()}] ${message}`);

export const all = Promise.all.bind(Promise);
