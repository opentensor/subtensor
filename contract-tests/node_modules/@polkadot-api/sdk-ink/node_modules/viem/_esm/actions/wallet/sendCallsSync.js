import { sendCalls, } from './sendCalls.js';
import { waitForCallsStatus, } from './waitForCallsStatus.js';
/**
 * Requests the connected wallet to send a batch of calls, and waits for the calls to be included in a block.
 *
 * - Docs: https://viem.sh/docs/actions/wallet/sendCallsSync
 * - JSON-RPC Methods: [`wallet_sendCalls`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @returns Calls status. {@link SendCallsSyncReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { sendCalls } from 'viem/actions'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const status = await sendCallsSync(client, {
 *   account: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 *   calls: [
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: 69420n,
 *     },
 *   ],
 * })
 */
export async function sendCallsSync(client, parameters) {
    const { chain = client.chain } = parameters;
    const timeout = parameters.timeout ?? Math.max((chain?.blockTime ?? 0) * 3, 5_000);
    const result = await sendCalls(client, parameters);
    const status = await waitForCallsStatus(client, {
        ...parameters,
        id: result.id,
        timeout,
    });
    return status;
}
//# sourceMappingURL=sendCallsSync.js.map