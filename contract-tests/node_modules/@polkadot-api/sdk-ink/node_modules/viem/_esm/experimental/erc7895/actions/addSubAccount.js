import { numberToHex } from '../../../utils/index.js';
/**
 * Requests to add a Sub Account.
 *
 * - Docs: https://viem.sh/experimental/erc7895/addSubAccount
 * - JSON-RPC Methods: [`wallet_addSubAccount`](https://github.com/ethereum/ERCs/blob/abd1c9f4eda2d6ad06ade0e3af314637a27d1ee7/ERCS/erc-7895.md)
 *
 * @param client - Client to use
 * @param parameters - {@link AddSubAccountParameters}
 * @returns Sub Account. {@link AddSubAccountReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { addSubAccount } from 'viem/experimental/erc7895'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const response = await addSubAccount(client, {
 *   keys: [{ publicKey: '0x0000000000000000000000000000000000000000', type: 'address' }],
 *   type: 'create',
 * })
 */
export async function addSubAccount(client, parameters) {
    return client.request({
        method: 'wallet_addSubAccount',
        params: [
            {
                account: {
                    ...parameters,
                    ...(parameters.chainId
                        ? { chainId: numberToHex(parameters.chainId) }
                        : {}),
                },
                version: '1',
            },
        ],
    });
}
//# sourceMappingURL=addSubAccount.js.map