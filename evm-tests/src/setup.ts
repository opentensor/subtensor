
import { createClient, TypedApi, PolkadotClient, Binary } from 'polkadot-api';
import { SUB_LOCAL_URL } from "./config"
import { getWsProvider } from 'polkadot-api/ws-provider/web';

// export async function getClient(url: ClientUrlType) {
//     const provider = getWsProvider(url);
//     const client = createClient(provider);
//     return client
// }

let client: PolkadotClient | undefined = undefined

export async function getClient() {
    if (client === undefined) {
        const provider = getWsProvider(SUB_LOCAL_URL);
        client = createClient(provider);
    }
    return client;
}

before(async () => {
    // const provider = getWsProvider(SUB_LOCAL_URL);
    // client = createClient(provider);

});

after(() => {
    client?.destroy()
});

