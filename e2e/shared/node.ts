import { spawn, ChildProcess } from "node:child_process";

const SECOND = 1000;
const MINUTE = 60 * SECOND;

// Substrate CLI shortcuts that inject keystore keys automatically.
const SUBSTRATE_SHORTCUTS = new Set([
  "alice",
  "bob",
  "charlie",
  "dave",
  "eve",
  "ferdie",
  "one",
  "two",
]);

export type NodeOptions = {
  binaryPath: string;
  basePath: string;
  name: string;
  port: number;
  rpcPort: number;
  validator: boolean;
  chainSpec: string;
};

export type Node = {
  name: string;
  binaryPath: string;
  rpcPort: number;
  port: number;
  process: ChildProcess;
};

export const log = (message: string) => console.log(`[${new Date().toISOString()}] ${message}`);

export const startNode = (opts: NodeOptions): Node => {
  const nameArgs = SUBSTRATE_SHORTCUTS.has(opts.name) ? [`--${opts.name}`] : ["--name", opts.name];

  const process = spawn(opts.binaryPath, [
    ...nameArgs,
    ...["--chain", opts.chainSpec],
    ...["--base-path", opts.basePath],
    ...["--port", opts.port.toString()],
    ...["--rpc-port", opts.rpcPort.toString()],
    ...(opts.validator ? ["--validator"] : []),
    "--rpc-cors=all",
    "--allow-private-ipv4",
    "--discover-local",
    "--unsafe-force-node-key-generation",
  ]);

  let lastStderr = "";
  process.stderr?.on("data", (chunk: Buffer) => {
    lastStderr = chunk.toString();
  });
  process.on("error", (error) => console.error(`${opts.name} (error): ${error}`));
  process.on("close", (code) => {
    if (code !== 0 && code !== null) {
      log(`${opts.name}: process crashed with code ${code}. Last stderr: ${lastStderr}`);
    } else {
      log(`${opts.name}: process closed with code ${code}`);
    }
  });

  return {
    name: opts.name,
    binaryPath: opts.binaryPath,
    rpcPort: opts.rpcPort,
    port: opts.port,
    process,
  };
};

export const stop = (node: Node): Promise<void> => {
  return new Promise((resolve, reject) => {
    node.process.on("close", () => resolve());
    node.process.on("error", reject);

    if (!node.process.kill()) {
      reject(new Error(`Failed to stop ${node.name}`));
    }
  });
};

export const started = (node: Node, timeout = 60 * SECOND) => {
  const errorMessage = `${node.name} failed to start in time`;

  return innerEnsure(node, errorMessage, timeout, (data, ok) => {
    if (data.includes("💤 Idle")) {
      log(`${node.name}: started using ${node.binaryPath}`);
      ok();
    }
  });
};

export const peerCount = (node: Node, expectedPeers: number, timeout = 60 * SECOND) => {
  const errorMessage = `${node.name} failed to reach ${expectedPeers} peers in time`;

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

export const finalizedBlocks = (node: Node, expectedFinalized: number, timeout = 10 * MINUTE) => {
  const errorMessage = `${node.name} failed to reach ${expectedFinalized} finalized blocks in time`;

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

export function innerEnsure(
  node: Node,
  errorMessage: string,
  timeout: number,
  f: (data: string, ok: () => void) => void,
) {
  return new Promise<void>((resolve, reject) => {
    const id = setTimeout(() => reject(new Error(errorMessage)), timeout);

    const fn = (chunk: Buffer) => {
      const data = chunk.toString();
      f(data, () => {
        clearTimeout(id);
        node.process.stderr?.off("data", fn);
        resolve();
      });
    };

    node.process.stderr?.on("data", fn);
  });
}
