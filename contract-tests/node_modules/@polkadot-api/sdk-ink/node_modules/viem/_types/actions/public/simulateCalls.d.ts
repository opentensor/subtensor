import type { AbiStateMutability, Address, Narrow } from 'abitype';
import * as AbiFunction from 'ox/AbiFunction';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Account } from '../../types/account.js';
import type { Block } from '../../types/block.js';
import type { Calls } from '../../types/calls.js';
import type { Chain } from '../../types/chain.js';
import type { Log } from '../../types/log.js';
import type { Hex } from '../../types/misc.js';
import type { MulticallResults } from '../../types/multicall.js';
import type { StateOverride } from '../../types/stateOverride.js';
import { type EncodeFunctionDataErrorType } from '../../utils/abi/encodeFunctionData.js';
import { type CreateAccessListErrorType } from './createAccessList.js';
import { type SimulateBlocksErrorType, type SimulateBlocksParameters } from './simulateBlocks.js';
export type SimulateCallsParameters<calls extends readonly unknown[] = readonly unknown[], account extends Account | Address | undefined = Account | Address | undefined> = Omit<SimulateBlocksParameters, 'blocks' | 'returnFullTransactions'> & {
    /** Account attached to the calls (msg.sender). */
    account?: account | undefined;
    /** Calls to simulate. */
    calls: Calls<Narrow<calls>>;
    /** State overrides. */
    stateOverrides?: StateOverride | undefined;
    /** Whether to trace asset changes. */
    traceAssetChanges?: boolean | undefined;
};
export type SimulateCallsReturnType<calls extends readonly unknown[] = readonly unknown[]> = {
    /** Asset changes. */
    assetChanges: readonly {
        token: {
            address: Address;
            decimals?: number | undefined;
            symbol?: string | undefined;
        };
        value: {
            pre: bigint;
            post: bigint;
            diff: bigint;
        };
    }[];
    /** Block results. */
    block: Block;
    /** Call results. */
    results: MulticallResults<Narrow<calls>, true, {
        extraProperties: {
            data: Hex;
            gasUsed: bigint;
            logs?: Log[] | undefined;
        };
        error: Error;
        mutability: AbiStateMutability;
    }>;
};
export type SimulateCallsErrorType = AbiFunction.encodeData.ErrorType | AbiFunction.from.ErrorType | CreateAccessListErrorType | EncodeFunctionDataErrorType | SimulateBlocksErrorType | ErrorType;
/**
 * Simulates execution of a batch of calls.
 *
 * @param client - Client to use
 * @param parameters - {@link SimulateCallsParameters}
 * @returns Results. {@link SimulateCallsReturnType}
 *
 * @example
 * ```ts
 * import { createPublicClient, http, parseEther } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { simulateCalls } from 'viem/actions'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const result = await simulateCalls(client, {
 *   account: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *   calls: [{
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: parseEther('1'),
 *     },
 *   ]
 * })
 * ```
 */
export declare function simulateCalls<const calls extends readonly unknown[], chain extends Chain | undefined, account extends Account | Address | undefined = undefined>(client: Client<Transport, chain>, parameters: SimulateCallsParameters<calls, account>): Promise<SimulateCallsReturnType<calls>>;
//# sourceMappingURL=simulateCalls.d.ts.map