import { getAssets, } from '../actions/getAssets.js';
/**
 * A suite of ERC-7811 Wallet Actions.
 *
 * @example
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { erc7811Actions } from 'viem/experimental/erc7811'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 * }).extend(erc7811Actions())
 *
 * const response = await client.getAssets({
 *   account: '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266',
 * })
 */
export function erc7811Actions() {
    return (client) => {
        return {
            // @ts-expect-error
            getAssets: (...[parameters]) => getAssets(client, parameters),
        };
    };
}
//# sourceMappingURL=erc7811.js.map