import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import { BaseError } from '../../errors/base.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Chain } from '../../types/chain.js';
import { type ObserveErrorType } from '../../utils/observe.js';
import { type PollErrorType } from '../../utils/poll.js';
import { type WithRetryParameters } from '../../utils/promise/withRetry.js';
import { type GetCallsStatusErrorType, type GetCallsStatusReturnType } from './getCallsStatus.js';
export type WaitForCallsStatusParameters = {
    /**
     * The id of the call batch to wait for.
     */
    id: string;
    /**
     * Polling frequency (in ms). Defaults to the client's pollingInterval config.
     *
     * @default client.pollingInterval
     */
    pollingInterval?: number | undefined;
    /**
     * Number of times to retry if the call bundle failed.
     * @default 4 (exponential backoff)
     */
    retryCount?: WithRetryParameters['retryCount'] | undefined;
    /**
     * Time to wait (in ms) between retries.
     * @default `({ count }) => ~~(1 << count) * 200` (exponential backoff)
     */
    retryDelay?: WithRetryParameters['delay'] | undefined;
    /**
     * The status range to wait for.
     *
     * @default (status) => status >= 200
     */
    status?: ((parameters: GetCallsStatusReturnType) => boolean) | undefined;
    /**
     * Whether to throw an error if the call bundle fails.
     *
     * @default false
     */
    throwOnFailure?: boolean | undefined;
    /**
     * Optional timeout (in milliseconds) to wait before stopping polling.
     *
     * @default 60_000
     */
    timeout?: number | undefined;
};
export type WaitForCallsStatusReturnType = GetCallsStatusReturnType;
export type WaitForCallsStatusErrorType = ObserveErrorType | PollErrorType | GetCallsStatusErrorType | WaitForCallsStatusTimeoutError | ErrorType;
/**
 * Waits for the status & receipts of a call bundle that was sent via `sendCalls`.
 *
 * - Docs: https://viem.sh/docs/actions/wallet/waitForCallsStatus
 * - JSON-RPC Methods: [`wallet_getCallsStatus`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @param parameters - {@link WaitForCallsStatusParameters}
 * @returns Status & receipts of the call bundle. {@link WaitForCallsStatusReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { waitForCallsStatus } from 'viem/actions'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 *
 * const { receipts, status } = await waitForCallsStatus(client, { id: '0xdeadbeef' })
 */
export declare function waitForCallsStatus<chain extends Chain | undefined>(client: Client<Transport, chain>, parameters: WaitForCallsStatusParameters): Promise<WaitForCallsStatusReturnType>;
export type WaitForCallsStatusTimeoutErrorType = WaitForCallsStatusTimeoutError & {
    name: 'WaitForCallsStatusTimeoutError';
};
export declare class WaitForCallsStatusTimeoutError extends BaseError {
    constructor({ id }: {
        id: string;
    });
}
//# sourceMappingURL=waitForCallsStatus.d.ts.map