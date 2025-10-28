
import { createClient, TypedApi, PolkadotClient, Binary } from 'polkadot-api';
import { SUB_LOCAL_URL } from "./config"
import { getWsProvider } from 'polkadot-api/ws-provider/web';
import { withPolkadotSdkCompat } from "polkadot-api/polkadot-sdk-compat"

let client: PolkadotClient | undefined = undefined

export async function getClient() {
    if (client === undefined) {
        const provider = getWsProvider(SUB_LOCAL_URL);
        client = createClient(provider);
    }
    return client;
}

export async function getSdkClient() {
    const client = createClient(
        withPolkadotSdkCompat(
            getWsProvider(SUB_LOCAL_URL),
        ),
    )
    return client
}

after(() => {
    client?.destroy()
});

