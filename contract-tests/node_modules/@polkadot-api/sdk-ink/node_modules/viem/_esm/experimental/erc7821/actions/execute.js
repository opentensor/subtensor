import { sendTransaction, } from '../../../actions/wallet/sendTransaction.js';
import { withCache } from '../../../utils/promise/withCache.js';
import { executionMode } from '../constants.js';
import { ExecuteUnsupportedError } from '../errors.js';
import { encodeExecuteData, } from '../utils/encodeExecuteData.js';
import { getExecuteError, } from '../utils/getExecuteError.js';
import { supportsExecutionMode } from './supportsExecutionMode.js';
/**
 * Executes call(s) using the `execute` function on an [ERC-7821-compatible contract](https://eips.ethereum.org/EIPS/eip-7821).
 *
 * @example
 * ```ts
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { execute } from 'viem/experimental/erc7821'
 *
 * const account = privateKeyToAccount('0x...')
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const hash = await execute(client, {
 *   account,
 *   calls: [{
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: 69420n,
 *     },
 *   }],
 *   to: account.address,
 * })
 * ```
 *
 * @example
 * ```ts
 * // Account Hoisting
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { execute } from 'viem/experimental/erc7821'
 *
 * const account = privateKeyToAccount('0x...')
 *
 * const client = createClient({
 *   account,
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const hash = await execute(client, {
 *   calls: [{
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: 69420n,
 *     },
 *   }],
 *   to: account.address,
 * })
 * ```
 *
 * @param client - Client to use.
 * @param parameters - {@link ExecuteParameters}
 * @returns Transaction hash. {@link ExecuteReturnType}
 */
export async function execute(client, parameters) {
    const { authorizationList, calls, opData } = parameters;
    const address = authorizationList?.[0]?.address ?? parameters.address;
    const mode = opData ? executionMode.opData : executionMode.default;
    const supported = await withCache(() => supportsExecutionMode(client, {
        address,
        mode,
    }), {
        cacheKey: `supportsExecutionMode.${client.uid}.${address}.${mode}`,
    });
    if (!supported)
        throw new ExecuteUnsupportedError();
    try {
        return await sendTransaction(client, {
            ...parameters,
            to: parameters.address,
            data: encodeExecuteData({ calls, opData }),
        });
    }
    catch (e) {
        throw getExecuteError(e, { calls });
    }
}
//# sourceMappingURL=execute.js.map