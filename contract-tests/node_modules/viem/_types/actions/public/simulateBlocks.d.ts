import type { AbiStateMutability, Address, Narrow } from 'abitype';
import * as BlockOverrides from 'ox/BlockOverrides';
import { type ParseAccountErrorType } from '../../accounts/utils/parseAccount.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { ErrorType } from '../../errors/utils.js';
import type { Account } from '../../types/account.js';
import type { Block, BlockTag } from '../../types/block.js';
import type { Calls } from '../../types/calls.js';
import type { Chain } from '../../types/chain.js';
import type { Log } from '../../types/log.js';
import type { Hex } from '../../types/misc.js';
import type { MulticallResults } from '../../types/multicall.js';
import type { StateOverride } from '../../types/stateOverride.js';
import type { TransactionRequest } from '../../types/transaction.js';
import type { ExactPartial, UnionOmit } from '../../types/utils.js';
import { type DecodeFunctionResultErrorType } from '../../utils/abi/decodeFunctionResult.js';
import { type EncodeFunctionDataErrorType } from '../../utils/abi/encodeFunctionData.js';
import { type NumberToHexErrorType } from '../../utils/encoding/toHex.js';
import { type GetNodeErrorReturnType } from '../../utils/errors/getNodeError.js';
import { type FormatBlockErrorType } from '../../utils/formatters/block.js';
import { type FormatTransactionRequestErrorType } from '../../utils/formatters/transactionRequest.js';
import { type SerializeStateOverrideErrorType } from '../../utils/stateOverride.js';
import { type AssertRequestErrorType } from '../../utils/transaction/assertRequest.js';
type CallExtraProperties = ExactPartial<UnionOmit<TransactionRequest, 'blobs' | 'data' | 'kzg' | 'to' | 'sidecars' | 'value'>> & {
    /** Account attached to the call (msg.sender). */
    account?: Account | Address | undefined;
};
export type SimulateBlocksParameters<calls extends readonly unknown[] = readonly unknown[]> = {
    /** Blocks to simulate. */
    blocks: readonly {
        /** Block overrides. */
        blockOverrides?: BlockOverrides.BlockOverrides | undefined;
        /** Calls to execute. */
        calls: Calls<Narrow<calls>, CallExtraProperties>;
        /** State overrides. */
        stateOverrides?: StateOverride | undefined;
    }[];
    /** Whether to return the full transactions. */
    returnFullTransactions?: boolean | undefined;
    /** Whether to trace transfers. */
    traceTransfers?: boolean | undefined;
    /** Whether to enable validation mode. */
    validation?: boolean | undefined;
} & ({
    /** The balance of the account at a block number. */
    blockNumber?: bigint | undefined;
    blockTag?: undefined;
} | {
    blockNumber?: undefined;
    /**
     * The balance of the account at a block tag.
     * @default 'latest'
     */
    blockTag?: BlockTag | undefined;
});
export type SimulateBlocksReturnType<calls extends readonly unknown[] = readonly unknown[]> = readonly (Block & {
    calls: MulticallResults<Narrow<calls>, true, {
        extraProperties: {
            data: Hex;
            gasUsed: bigint;
            logs?: Log[] | undefined;
        };
        error: Error;
        mutability: AbiStateMutability;
    }>;
})[];
export type SimulateBlocksErrorType = AssertRequestErrorType | DecodeFunctionResultErrorType | EncodeFunctionDataErrorType | FormatBlockErrorType | FormatTransactionRequestErrorType | GetNodeErrorReturnType | ParseAccountErrorType | SerializeStateOverrideErrorType | NumberToHexErrorType | ErrorType;
/**
 * Simulates a set of calls on block(s) with optional block and state overrides.
 *
 * @example
 * ```ts
 * import { createClient, http, parseEther } from 'viem'
 * import { simulate } from 'viem/actions'
 * import { mainnet } from 'viem/chains'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const result = await simulate(client, {
 *   blocks: [{
 *     blockOverrides: {
 *       number: 69420n,
 *     },
 *     calls: [{
 *       {
 *         account: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *         data: '0xdeadbeef',
 *         to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       },
 *       {
 *         account: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *         to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *         value: parseEther('1'),
 *       },
 *     }],
 *     stateOverrides: [{
 *       address: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *       balance: parseEther('10'),
 *     }],
 *   }]
 * })
 * ```
 *
 * @param client - Client to use.
 * @param parameters - {@link SimulateBlocksParameters}
 * @returns Simulated blocks. {@link SimulateBlocksReturnType}
 */
export declare function simulateBlocks<chain extends Chain | undefined, const calls extends readonly unknown[]>(client: Client<Transport, chain>, parameters: SimulateBlocksParameters<calls>): Promise<SimulateBlocksReturnType<calls>>;
export {};
//# sourceMappingURL=simulateBlocks.d.ts.map