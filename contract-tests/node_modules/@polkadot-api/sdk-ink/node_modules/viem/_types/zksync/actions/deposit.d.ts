import { type Address } from 'abitype';
import type { Account } from '../../accounts/types.js';
import { type SendTransactionErrorType, type SendTransactionReturnType } from '../../actions/wallet/sendTransaction.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { GetAccountParameter } from '../../types/account.js';
import type { Chain, DeriveChain, GetChainParameter } from '../../types/chain.js';
import type { Hex } from '../../types/misc.js';
import type { UnionEvaluate, UnionOmit } from '../../types/utils.js';
import { type FormattedTransactionRequest } from '../../utils/index.js';
import { type BaseFeeHigherThanValueErrorType } from '../errors/bridge.js';
import type { ChainEIP712 } from '../types/chain.js';
export type DepositParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>> = UnionEvaluate<UnionOmit<FormattedTransactionRequest<_derivedChain>, 'data' | 'to' | 'from'>> & Partial<GetChainParameter<chain, chainOverride>> & Partial<GetAccountParameter<account>> & {
    /** L2 client. */
    client: Client<Transport, chainL2, accountL2>;
    /** The address of the token to deposit. */
    token: Address;
    /** The amount of the token to deposit. */
    amount: bigint;
    /** The address that will receive the deposited tokens on L2.
    Defaults to the sender address.*/
    to?: Address | undefined;
    /** (currently not used) The tip the operator will receive on top of
    the base cost of the transaction. */
    operatorTip?: bigint | undefined;
    /** Maximum amount of L2 gas that transaction can consume during execution on L2. */
    l2GasLimit?: bigint | undefined;
    /** The L2 gas price for each published L1 calldata byte. */
    gasPerPubdataByte?: bigint | undefined;
    /** The address on L2 that will receive the refund for the transaction.
    If the transaction fails, it will also be the address to receive `amount`. */
    refundRecipient?: Address | undefined;
    /** The address of the bridge contract to be used.
    Defaults to the default ZKsync L1 shared bridge. */
    bridgeAddress?: Address | undefined;
    /** Additional data that can be sent to a bridge. */
    customBridgeData?: Hex | undefined;
    /** Whether token approval should be performed under the hood.
    Set this flag to true (or provide transaction overrides) if the bridge does
    not have sufficient allowance. The approval transaction is executed only if
    the bridge lacks sufficient allowance; otherwise, it is skipped. */
    approveToken?: boolean | UnionEvaluate<UnionOmit<FormattedTransactionRequest<_derivedChain>, 'data' | 'to' | 'from'>> | undefined;
    /** Whether base token approval should be performed under the hood.
    Set this flag to true (or provide transaction overrides) if the bridge does
    not have sufficient allowance. The approval transaction is executed only if
    the bridge lacks sufficient allowance; otherwise, it is skipped. */
    approveBaseToken?: boolean | UnionEvaluate<UnionOmit<FormattedTransactionRequest<_derivedChain>, 'data' | 'to' | 'from'>> | undefined;
};
export type DepositReturnType = SendTransactionReturnType;
export type DepositErrorType = SendTransactionErrorType | BaseFeeHigherThanValueErrorType;
/**
 * Transfers the specified token from the associated account on the L1 network to the target account on the L2 network.
 * The token can be either ETH or any ERC20 token. For ERC20 tokens, enough approved tokens must be associated with
 * the specified L1 bridge (default one or the one defined in `bridgeAddress`).
 * In this case, depending on is the chain ETH-based or not `approveToken` or `approveBaseToken`
 * can be enabled to perform token approval. If there are already enough approved tokens for the L1 bridge,
 * token approval will be skipped.
 *
 * @param client - Client to use
 * @param parameters - {@link DepositParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link DepositReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { deposit, legacyEthAddress, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *     chain: mainnet,
 *     transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const account = privateKeyToAccount('0x…')
 *
 * const hash = await deposit(client, {
 *     client: clientL2,
 *     account,
 *     token: legacyEthAddress,
 *     to: account.address,
 *     amount: 1_000_000_000_000_000_000n,
 *     refundRecipient: account.address,
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { legacyEthAddress, publicActionsL2 } from 'viem/zksync'
 *
 * const walletClient = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 *   account: privateKeyToAccount('0x…'),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await deposit(walletClient, {
 *     client: clientL2,
 *     token: legacyEthAddress,
 *     to: walletClient.account.address,
 *     amount: 1_000_000_000_000_000_000n,
 *     refundRecipient: walletClient.account.address,
 * })
 */
export declare function deposit<chain extends Chain | undefined, account extends Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>>(client: Client<Transport, chain, account>, parameters: DepositParameters<chain, account, chainOverride, chainL2, accountL2>): Promise<DepositReturnType>;
//# sourceMappingURL=deposit.d.ts.map