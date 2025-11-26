import { execute, } from '../actions/execute.js';
import { executeBatches, } from '../actions/executeBatches.js';
import { supportsExecutionMode, } from '../actions/supportsExecutionMode.js';
/**
 * A suite of Actions for [ERC-7821](https://eips.ethereum.org/EIPS/eip-7821).
 *
 * @example
 * import { createClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { erc7821Actions } from 'viem/experimental'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * }).extend(erc7821Actions())
 */
export function erc7821Actions() {
    return (client) => {
        return {
            execute: (parameters) => execute(client, parameters),
            executeBatches: (parameters) => executeBatches(client, parameters),
            supportsExecutionMode: (parameters) => supportsExecutionMode(client, parameters),
        };
    };
}
//# sourceMappingURL=erc7821.js.map