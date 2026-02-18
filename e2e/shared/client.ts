import { DedotClient, WsProvider } from "dedot";
import { Keyring } from "@polkadot/keyring";
import type { NodeSubtensorApi } from "../node-subtensor/index.js";

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
