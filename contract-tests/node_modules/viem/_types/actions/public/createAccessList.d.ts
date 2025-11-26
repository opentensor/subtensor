import type { Address } from 'abitype';
import type { Account } from '../../accounts/types.js';
import { type ParseAccountErrorType } from '../../accounts/utils/parseAccount.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { BlockTag } from '../../types/block.js';
import type { Chain } from '../../types/chain.js';
import type { AccessList } from '../../types/transaction.js';
import type { Prettify, UnionOmit } from '../../types/utils.js';
import type { RequestErrorType } from '../../utils/buildRequest.js';
import { type NumberToHexErrorType } from '../../utils/encoding/toHex.js';
import { type GetCallErrorReturnType } from '../../utils/errors/getCallError.js';
import { type FormatTransactionRequestErrorType, type FormattedTransactionRequest } from '../../utils/formatters/transactionRequest.js';
import type { AssertRequestErrorType } from '../../utils/transaction/assertRequest.js';
export type CreateAccessListParameters<chain extends Chain | undefined = Chain | undefined> = UnionOmit<FormattedTransactionRequest<chain>, 'from' | 'nonce' | 'accessList'> & {
    /** Account attached to the call (msg.sender). */
    account?: Account | Address | undefined;
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
export type CreateAccessListReturnType = Prettify<{
    accessList: AccessList;
    gasUsed: bigint;
}>;
export type CreateAccessListErrorType = GetCallErrorReturnType<ParseAccountErrorType | AssertRequestErrorType | NumberToHexErrorType | FormatTransactionRequestErrorType | RequestErrorType>;
/**
 * Creates an EIP-2930 access list.
 *
 * - Docs: https://viem.sh/docs/actions/public/createAccessList
 * - JSON-RPC Methods: `eth_createAccessList`
 *
 * @param client - Client to use
 * @param parameters - {@link CreateAccessListParameters}
 * @returns The access list. {@link CreateAccessListReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { createAccessList } from 'viem/public'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const data = await createAccessList(client, {
 *   account: '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266',
 *   data: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 * })
 */
export declare function createAccessList<chain extends Chain | undefined>(client: Client<Transport, chain>, args: CreateAccessListParameters<chain>): Promise<CreateAccessListReturnType>;
//# sourceMappingURL=createAccessList.d.ts.map