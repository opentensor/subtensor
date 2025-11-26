import { readContract } from '../../actions/public/readContract.js';
import { sendTransaction, } from '../../actions/wallet/sendTransaction.js';
import { AccountNotFoundError } from '../../errors/account.js';
import { ChainNotFoundError, } from '../../errors/chain.js';
import { decodeAbiParameters, encodeFunctionData, isAddressEqual, parseAccount, slice, } from '../../utils/index.js';
import { l1SharedBridgeAbi, l2SharedBridgeAbi } from '../constants/abis.js';
import { l2BaseTokenAddress } from '../constants/address.js';
import { WithdrawalLogNotFoundError, } from '../errors/bridge.js';
import { getWithdrawalL2ToL1Log } from '../utils/bridge/getWithdrawalL2ToL1Log.js';
import { getWithdrawalLog } from '../utils/bridge/getWithdrawalLog.js';
import { getDefaultBridgeAddresses } from './getDefaultBridgeAddresses.js';
import { getLogProof } from './getLogProof.js';
/**
 * Proves the inclusion of the `L2->L1` withdrawal message.
 *
 * @param client - Client to use
 * @param parameters - {@link FinalizeWithdrawalParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link FinalizeWithdrawalReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet, zksync } from 'viem/chains'
 * import { finalizeWithdrawal, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await finalizeWithdrawal(client, {
 *     account: privateKeyToAccount('0x…'),
 *     client: clientL2,
 *     hash: '0x...',
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet, zksync } from 'viem/chains'
 * import { finalizeWithdrawal, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createWalletClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await finalizeWithdrawal(client, {
 *     client: clientL2,
 *     hash: '0x…',
 * })
 */
export async function finalizeWithdrawal(client, parameters) {
    const { account: account_ = client.account, client: l2Client, hash, index = 0, ...rest } = parameters;
    const account = account_ ? parseAccount(account_) : client.account;
    if (!account)
        throw new AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new ChainNotFoundError();
    const { l1BatchNumber, l2MessageIndex, l2TxNumberInBlock, message, sender, proof, } = await getFinalizeWithdrawalParams(l2Client, { hash, index });
    let l1Bridge;
    if (isAddressEqual(sender, l2BaseTokenAddress))
        l1Bridge = (await getDefaultBridgeAddresses(l2Client)).sharedL1;
    else if (!(await isLegacyBridge(l2Client, { address: sender })))
        l1Bridge = await readContract(l2Client, {
            address: sender,
            abi: l2SharedBridgeAbi,
            functionName: 'l1SharedBridge',
            args: [],
        });
    else
        l1Bridge = await readContract(l2Client, {
            address: sender,
            abi: l2SharedBridgeAbi,
            functionName: 'l1Bridge',
            args: [],
        });
    const data = encodeFunctionData({
        abi: l1SharedBridgeAbi,
        functionName: 'finalizeWithdrawal',
        args: [
            BigInt(l2Client.chain.id),
            l1BatchNumber,
            BigInt(l2MessageIndex),
            Number(l2TxNumberInBlock),
            message,
            proof,
        ],
    });
    return await sendTransaction(client, {
        account,
        to: l1Bridge,
        data,
        value: 0n,
        ...rest,
    });
}
async function getFinalizeWithdrawalParams(client, parameters) {
    const { hash } = parameters;
    const { log, l1BatchTxId } = await getWithdrawalLog(client, parameters);
    const { l2ToL1LogIndex } = await getWithdrawalL2ToL1Log(client, parameters);
    const sender = slice(log.topics[1], 12);
    const proof = await getLogProof(client, {
        txHash: hash,
        index: l2ToL1LogIndex,
    });
    if (!proof) {
        throw new WithdrawalLogNotFoundError({ hash });
    }
    const [message] = decodeAbiParameters([{ type: 'bytes' }], log.data);
    return {
        l1BatchNumber: log.l1BatchNumber,
        l2MessageIndex: proof.id,
        l2TxNumberInBlock: l1BatchTxId,
        message,
        sender,
        proof: proof.proof,
    };
}
async function isLegacyBridge(client, parameters) {
    try {
        await readContract(client, {
            address: parameters.address,
            abi: l2SharedBridgeAbi,
            functionName: 'l1SharedBridge',
            args: [],
        });
        return false;
    }
    catch (_e) {
        return true;
    }
}
//# sourceMappingURL=finalizeWithdrawal.js.map