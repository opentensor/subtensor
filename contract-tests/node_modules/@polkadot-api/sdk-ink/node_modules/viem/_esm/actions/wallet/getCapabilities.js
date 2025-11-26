import { parseAccount } from '../../accounts/utils/parseAccount.js';
import { numberToHex } from '../../utils/encoding/toHex.js';
/**
 * Extract capabilities that a connected wallet supports (e.g. paymasters, session keys, etc).
 *
 * - Docs: https://viem.sh/docs/actions/wallet/getCapabilities
 * - JSON-RPC Methods: [`wallet_getCapabilities`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @returns The wallet's capabilities. {@link GetCapabilitiesReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { getCapabilities } from 'viem/actions'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const capabilities = await getCapabilities(client)
 */
export async function getCapabilities(client, parameters = {}) {
    const { account = client.account, chainId } = parameters;
    const account_ = account ? parseAccount(account) : undefined;
    const params = chainId
        ? [account_?.address, [numberToHex(chainId)]]
        : [account_?.address];
    const capabilities_raw = await client.request({
        method: 'wallet_getCapabilities',
        params,
    });
    const capabilities = {};
    for (const [chainId, capabilities_] of Object.entries(capabilities_raw)) {
        capabilities[Number(chainId)] = {};
        for (let [key, value] of Object.entries(capabilities_)) {
            if (key === 'addSubAccount')
                key = 'unstable_addSubAccount';
            capabilities[Number(chainId)][key] = value;
        }
    }
    return (typeof chainId === 'number' ? capabilities[chainId] : capabilities);
}
//# sourceMappingURL=getCapabilities.js.map