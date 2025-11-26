import { readContract } from '../../../actions/public/readContract.js';
import { withCache } from '../../../utils/promise/withCache.js';
import { abi, executionMode } from '../constants.js';
const toSerializedMode = {
    default: executionMode.default,
    opData: executionMode.opData,
    batchOfBatches: executionMode.batchOfBatches,
};
/**
 * Checks if the contract supports the ERC-7821 execution mode.
 *
 * @example
 * ```ts
 * import { createClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { supportsExecutionMode } from 'viem/experimental/erc7821'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const supported = await supportsExecutionMode(client, {
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 * })
 * ```
 *
 * @param client - Client to use.
 * @param parameters - {@link SupportsExecutionModeParameters}
 * @returns If the execution mode is supported. {@link SupportsExecutionModeReturnType}
 */
export async function supportsExecutionMode(client, parameters) {
    const { address, mode: m = 'default' } = parameters;
    const mode = m.startsWith('0x') ? m : toSerializedMode[m];
    try {
        return await withCache(() => readContract(client, {
            abi,
            address,
            functionName: 'supportsExecutionMode',
            args: [mode],
        }), {
            cacheKey: `supportsExecutionMode.${address}.${mode}`,
        });
    }
    catch {
        return false;
    }
}
//# sourceMappingURL=supportsExecutionMode.js.map