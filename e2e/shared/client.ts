import { DedotClient, WsProvider } from "dedot";
import { Keyring } from "@polkadot/keyring";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { NodeSubtensorApi } from "../node-subtensor/index.js";

const SECOND = 1000;

export const connectClient = async (rpcPort: number): Promise<DedotClient<NodeSubtensorApi>> => {
  const provider = new WsProvider(`ws://localhost:${rpcPort}`);
  return DedotClient.new<NodeSubtensorApi>(provider);
};

export const createKeyring = () => {
  return new Keyring({ type: "sr25519" });
};

export const getAccountNonce = async (
  client: { query: { system: { account: (address: string) => Promise<{ nonce: number }> } } },
  address: string,
): Promise<number> => {
  const account = await client.query.system.account(address);
  return account.nonce;
};

export const getBalance = async (
  client: {
    query: { system: { account: (address: string) => Promise<{ data: { free: bigint } }> } };
  },
  address: string,
): Promise<bigint> => {
  const account = await client.query.system.account(address);
  return account.data.free;
};

export const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** Polls the chain until `count` new finalized blocks have been produced. */
export async function waitForFinalizedBlocks(
  client: {
    rpc: {
      chain_getFinalizedHead: () => Promise<`0x${string}`>;
      chain_getHeader: (hash: `0x${string}`) => Promise<{ number: number } | undefined>;
    };
  },
  count: number,
  pollInterval = 1 * SECOND,
  timeout = 120 * SECOND,
): Promise<void> {
  const startHash = await client.rpc.chain_getFinalizedHead();
  const startHeader = await client.rpc.chain_getHeader(startHash);
  const start = startHeader!.number;
  const target = start + count;
  const deadline = Date.now() + timeout;

  while (Date.now() < deadline) {
    await sleep(pollInterval);
    const hash = await client.rpc.chain_getFinalizedHead();
    const header = await client.rpc.chain_getHeader(hash);
    if (header && header.number >= target) return;
  }

  throw new Error(
    `Timed out waiting for ${count} finalized blocks (from #${start}, target #${target})`,
  );
}

type TxStatus = { type: string; value?: { error?: string } };

/** Signs, sends, and watches a transaction until one of the given terminal
 *  status types is observed. If signAndSend itself rejects (e.g. pool
 *  rejection), the error is wrapped as an Invalid status. */
export const watchTxStatus = (
  tx: any,
  signer: KeyringPair,
  options: Record<string, unknown>,
  terminalTypes: string[],
  timeout = 30_000,
): Promise<TxStatus> => {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(`watchTxStatus timed out waiting for ${terminalTypes.join("/")}`)),
      timeout,
    );

    tx.signAndSend(signer, options, (result: { status: TxStatus }) => {
      if (terminalTypes.includes(result.status.type)) {
        clearTimeout(timer);
        resolve(result.status);
      }
    }).catch((err: unknown) => {
      clearTimeout(timer);
      resolve({ type: "Invalid", value: { error: String(err) } });
    });
  });
};
