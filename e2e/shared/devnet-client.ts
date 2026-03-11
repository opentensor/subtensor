import { subtensor } from "@polkadot-api/descriptors";
import { TypedApi, PolkadotClient, createClient } from "polkadot-api";
import { getWsProvider } from "polkadot-api/ws-provider/web";

let client: PolkadotClient | undefined = undefined;
let api: TypedApi<typeof subtensor> | undefined = undefined;

export async function getClient(rpcUrl: string): Promise<PolkadotClient> {
  if (client === undefined) {
    const provider = getWsProvider(rpcUrl);
    client = createClient(provider);
  }
  return client;
}

export async function getDevnetApi(rpcUrl: string): Promise<TypedApi<typeof subtensor>> {
  if (api === undefined) {
    const c = await getClient(rpcUrl);
    api = c.getTypedApi(subtensor);
  }
  return api;
}

export function destroyClient(): void {
  client?.destroy();
  client = undefined;
  api = undefined;
}
