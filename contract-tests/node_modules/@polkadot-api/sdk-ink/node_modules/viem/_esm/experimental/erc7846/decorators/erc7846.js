import { connect, } from '../actions/connect.js';
import { disconnect } from '../actions/disconnect.js';
/**
 * A suite of ERC-7846 Wallet Actions.
 *
 * @example
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { erc7846Actions } from 'viem/experimental/erc7846'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 * }).extend(erc7846Actions())
 *
 * const response = await client.connect()
 */
export function erc7846Actions() {
    return (client) => {
        return {
            connect: (parameters) => connect(client, parameters),
            disconnect: () => disconnect(client),
        };
    };
}
//# sourceMappingURL=erc7846.js.map