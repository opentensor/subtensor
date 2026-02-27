import { subtensor } from "@polkadot-api/descriptors";
import { TypedApi, PolkadotClient, createClient } from "polkadot-api";
import { getWsProvider } from "polkadot-api/ws-provider/web";

export const SUB_LOCAL_URL = "ws://localhost:9944";

let client: PolkadotClient | undefined = undefined;
let api: TypedApi<typeof subtensor> | undefined = undefined;

export async function getClient(): Promise<PolkadotClient> {
  if (client === undefined) {
    const provider = getWsProvider(SUB_LOCAL_URL);
    client = createClient(provider);
  }
  return client;
}

export async function getDevnetApi(): Promise<TypedApi<typeof subtensor>> {
  if (api === undefined) {
    const c = await getClient();
    api = c.getTypedApi(subtensor);
  }
  return api;
}

export function destroyClient(): void {
  client?.destroy();
  client = undefined;
  api = undefined;
}
