import { addSubAccount, } from '../actions/addSubAccount.js';
/**
 * A suite of ERC-7895 Wallet Actions.
 *
 * @example
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { erc7895Actions } from 'viem/experimental'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 * }).extend(erc7895Actions())
 *
 * const response = await client.addSubAccount({
 *   keys: [{ key: '0x0000000000000000000000000000000000000000', type: 'address' }],
 *   type: 'create',
 * })
 */
export function erc7895Actions() {
    return (client) => {
        return {
            addSubAccount: (parameters) => addSubAccount(client, parameters),
        };
    };
}
//# sourceMappingURL=erc7895.js.map