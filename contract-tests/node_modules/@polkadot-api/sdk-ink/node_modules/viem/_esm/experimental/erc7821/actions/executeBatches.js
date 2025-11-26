import { sendTransaction, } from '../../../actions/wallet/sendTransaction.js';
import { withCache } from '../../../utils/promise/withCache.js';
import { ExecuteUnsupportedError } from '../errors.js';
import { encodeExecuteBatchesData, } from '../utils/encodeExecuteBatchesData.js';
import { getExecuteError, } from '../utils/getExecuteError.js';
import { supportsExecutionMode } from './supportsExecutionMode.js';
/**
 * Executes batches of call(s) using "batch of batches" mode on an [ERC-7821-compatible contract](https://eips.ethereum.org/EIPS/eip-7821).
 *
 * @example
 * ```ts
 * import { createClient, http, parseEther } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { executeBatches } from 'viem/experimental/erc7821'
 *
 * const account = privateKeyToAccount('0x...')
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const hash = await executeBatches(client, {
 *   account,
 *   batches: [
 *     {
 *       calls: [
 *         {
 *           to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *           value: parseEther('1'),
 *         },
 *       ],
 *     },
 *     {
 *       calls: [
 *         {
 *           to: '0xcb98643b8786950F0461f3B0edf99D88F274574D',
 *           value: parseEther('2'),
 *         },
 *         {
 *           data: '0xdeadbeef',
 *           to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *         },
 *       ],
 *     },
 *   ],
 *   to: account.address,
 * })
 * ```
 *
 * @example
 * ```ts
 * // Account Hoisting
 * import { createClient, http, parseEther } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { executeBatches } from 'viem/experimental/erc7821'
 *
 * const account = privateKeyToAccount('0x...')
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const hash = await executeBatches(client, {
 *   batches: [
 *     {
 *       calls: [
 *         {
 *           to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *           value: parseEther('1'),
 *         },
 *       ],
 *     },
 *     {
 *       calls: [
 *         {
 *           to: '0xcb98643b8786950F0461f3B0edf99D88F274574D',
 *           value: parseEther('2'),
 *         },
 *         {
 *           data: '0xdeadbeef',
 *           to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *         },
 *       ],
 *     },
 *   ],
 *   to: account.address,
 * })
 * ```
 *
 * @param client - Client to use.
 * @param parameters - {@link ExecuteBatchesParameters}
 * @returns Transaction hash. {@link ExecuteBatchesReturnType}
 */
export async function executeBatches(client, parameters) {
    const { authorizationList, batches } = parameters;
    const address = authorizationList?.[0]?.address ?? parameters.address;
    const supported = await withCache(() => supportsExecutionMode(client, {
        address,
        mode: 'batchOfBatches',
    }), {
        cacheKey: `supportsExecutionMode.${client.uid}.${address}.batchOfBatches`,
    });
    if (!supported)
        throw new ExecuteUnsupportedError();
    try {
        return await sendTransaction(client, {
            ...parameters,
            to: parameters.address,
            data: encodeExecuteBatchesData({ batches }),
        });
    }
    catch (e) {
        const calls = batches.flatMap((b) => b.calls);
        throw getExecuteError(e, { calls });
    }
}
//# sourceMappingURL=executeBatches.js.map