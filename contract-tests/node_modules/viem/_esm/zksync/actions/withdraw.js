import { AccountNotFoundError } from '../../errors/account.js';
import { encodeFunctionData, isAddressEqual, parseAccount, } from '../../utils/index.js';
import { ethTokenAbi, l2SharedBridgeAbi } from '../constants/abis.js';
import { ethAddressInContracts, l2BaseTokenAddress, legacyEthAddress, } from '../constants/address.js';
import { getDefaultBridgeAddresses } from './getDefaultBridgeAddresses.js';
import { getL2TokenAddress } from './getL2TokenAddress.js';
import { sendTransaction, } from './sendTransaction.js';
/**
 * Initiates the withdrawal process which withdraws ETH or any ERC20 token
 * from the associated account on L2 network to the target account on L1 network.
 *
 * @param client - Client to use
 * @param parameters - {@link WithdrawParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link WithdrawReturnType}
 *
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync } from 'viem/chains'
 * import { withdraw, legacyEthAddress } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await withdraw(client, {
 *     account: privateKeyToAccount('0x…'),
 *     amount: 1_000_000_000_000_000_000n,
 *     token: legacyEthAddress,
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync } from 'viem/chains'
 * import { withdraw, legacyEthAddress } from 'viem/zksync'
 *
 * const client = createWalletClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await withdraw(client, {
 *     amount: 1_000_000_000_000_000_000n,
 *     token: legacyEthAddress,
 * })
 *
 * @example Paymaster
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync } from 'viem/chains'
 * import {
 *   withdraw,
 *   legacyEthAddress,
 *   getApprovalBasedPaymasterInput
 * } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await withdraw(client, {
 *     account: privateKeyToAccount('0x…'),
 *     amount: 1_000_000_000_000_000_000n,
 *     token: legacyEthAddress,
 *     paymaster: '0x0EEc6f45108B4b806e27B81d9002e162BD910670',
 *     paymasterInput: getApprovalBasedPaymasterInput({
 *       minAllowance: 1n,
 *       token: '0x2dc3685cA34163952CF4A5395b0039c00DFa851D',
 *       innerInput: new Uint8Array(),
 *     }),
 * })
 */
export async function withdraw(client, parameters) {
    let { account: account_ = client.account, token = l2BaseTokenAddress, to, amount, bridgeAddress, ...rest } = parameters;
    const account = account_ ? parseAccount(account_) : client.account;
    if (!account)
        throw new AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!to)
        to = account.address;
    let data;
    let contract;
    let value = 0n;
    if (isAddressEqual(token, legacyEthAddress) ||
        isAddressEqual(token, ethAddressInContracts))
        token = await getL2TokenAddress(client, { token: ethAddressInContracts });
    if (isAddressEqual(token, l2BaseTokenAddress)) {
        data = encodeFunctionData({
            abi: ethTokenAbi,
            functionName: 'withdraw',
            args: [to],
        });
        value = amount;
        contract = l2BaseTokenAddress;
    }
    else {
        data = encodeFunctionData({
            abi: l2SharedBridgeAbi,
            functionName: 'withdraw',
            args: [to, token, amount],
        });
        contract = bridgeAddress
            ? bridgeAddress
            : (await getDefaultBridgeAddresses(client)).sharedL2;
    }
    return await sendTransaction(client, {
        account,
        to: contract,
        data,
        value,
        ...rest,
    });
}
//# sourceMappingURL=withdraw.js.map