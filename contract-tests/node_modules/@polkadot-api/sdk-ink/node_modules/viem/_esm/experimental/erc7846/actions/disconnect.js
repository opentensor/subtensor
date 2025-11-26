/**
 * Requests to disconnect connected accounts.
 *
 * - Docs: https://viem.sh/experimental/erc7846/disconnect
 * - JSON-RPC Methods: [`wallet_disconnect`](https://github.com/ethereum/ERCs/blob/abd1c9f4eda2d6ad06ade0e3af314637a27d1ee7/ERCS/erc-7846.md)
 *
 * @param client - Client to use
 * @returns void
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { disconnect } from 'viem/experimental/erc7846'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 *
 * await disconnect(client)
 */
export async function disconnect(client) {
    return await client.request({ method: 'wallet_disconnect' }, { dedupe: true, retryCount: 0 });
}
//# sourceMappingURL=disconnect.js.map