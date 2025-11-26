import type { Address, Narrow } from 'abitype';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Account, GetAccountParameter } from '../../types/account.js';
import type { Calls } from '../../types/calls.js';
import type { ExtractCapabilities } from '../../types/capabilities.js';
import type { Chain, DeriveChain } from '../../types/chain.js';
import type { WalletSendCallsParameters } from '../../types/eip1193.js';
import type { Prettify } from '../../types/utils.js';
import type { RequestErrorType } from '../../utils/buildRequest.js';
export declare const fallbackMagicIdentifier = "0x5792579257925792579257925792579257925792579257925792579257925792";
export declare const fallbackTransactionErrorMagicIdentifier: `0x${string}`;
export type SendCallsParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, calls extends readonly unknown[] = readonly unknown[], _chain extends Chain | undefined = DeriveChain<chain, chainOverride>> = {
    chain?: chainOverride | Chain | undefined;
    calls: Calls<Narrow<calls>>;
    capabilities?: ExtractCapabilities<'sendCalls', 'Request'> | undefined;
    experimental_fallback?: boolean | undefined;
    experimental_fallbackDelay?: number | undefined;
    forceAtomic?: boolean | undefined;
    id?: string | undefined;
    version?: WalletSendCallsParameters[number]['version'] | undefined;
} & GetAccountParameter<account, Account | Address, false, true>;
export type SendCallsReturnType = Prettify<{
    capabilities?: ExtractCapabilities<'sendCalls', 'ReturnType'> | undefined;
    id: string;
}>;
export type SendCallsErrorType = RequestErrorType | ErrorType;
/**
 * Requests the connected wallet to send a batch of calls.
 *
 * - Docs: https://viem.sh/docs/actions/wallet/sendCalls
 * - JSON-RPC Methods: [`wallet_sendCalls`](https://eips.ethereum.org/EIPS/eip-5792)
 *
 * @param client - Client to use
 * @returns Transaction identifier. {@link SendCallsReturnType}
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
 * const id = await sendCalls(client, {
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
export declare function sendCalls<const calls extends readonly unknown[], chain extends Chain | undefined, account extends Account | undefined = undefined, chainOverride extends Chain | undefined = undefined>(client: Client<Transport, chain, account>, parameters: SendCallsParameters<chain, account, chainOverride, calls>): Promise<SendCallsReturnType>;
//# sourceMappingURL=sendCalls.d.ts.map