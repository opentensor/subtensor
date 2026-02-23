import { createClient, type PolkadotClient, type TypedApi } from "polkadot-api";
import { getWsProvider } from "polkadot-api/ws-provider";
import { getPolkadotSigner, type PolkadotSigner } from "polkadot-api/signer";
import { sr25519CreateDerive } from "@polkadot-labs/hdkd";
import {
  DEV_PHRASE,
  entropyToMiniSecret,
  mnemonicToEntropy,
  ss58Address,
} from "@polkadot-labs/hdkd-helpers";
import { subtensor } from "@polkadot-api/descriptors";

const SECOND = 1000;

export type ClientConnection = {
  client: PolkadotClient;
  api: TypedApi<typeof subtensor>;
};

export const connectClient = async (rpcPort: number): Promise<ClientConnection> => {
  const provider = getWsProvider(`ws://localhost:${rpcPort}`);
  const client = createClient(provider);
  const api = client.getTypedApi(subtensor);
  return { client, api };
};

export type Signer = {
  signer: PolkadotSigner;
  address: string;
};

export const createSigner = (uri: string): Signer => {
  const entropy = mnemonicToEntropy(DEV_PHRASE);
  const miniSecret = entropyToMiniSecret(entropy);
  const derive = sr25519CreateDerive(miniSecret);
  const keypair = derive(uri);
  return {
    signer: getPolkadotSigner(keypair.publicKey, "Sr25519", keypair.sign),
    address: ss58Address(keypair.publicKey),
  };
};

export const getAccountNonce = async (
  api: TypedApi<typeof subtensor>,
  address: string,
): Promise<number> => {
  const account = await api.query.System.Account.getValue(address, { at: "best" });
  return account.nonce;
};

export const getBalance = async (
  api: TypedApi<typeof subtensor>,
  address: string,
): Promise<bigint> => {
  const account = await api.query.System.Account.getValue(address);
  return account.data.free;
};

export const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** Polls the chain until `count` new finalized blocks have been produced. */
export async function waitForFinalizedBlocks(
  client: PolkadotClient,
  count: number,
  pollInterval = 1 * SECOND,
  timeout = 120 * SECOND,
): Promise<void> {
  const startBlock = await client.getFinalizedBlock();
  const start = startBlock.number;
  const target = start + count;
  const deadline = Date.now() + timeout;

  while (Date.now() < deadline) {
    await sleep(pollInterval);
    const block = await client.getFinalizedBlock();
    if (block.number >= target) return;
  }

  throw new Error(
    `Timed out waiting for ${count} finalized blocks (from #${start}, target #${target})`,
  );
}
