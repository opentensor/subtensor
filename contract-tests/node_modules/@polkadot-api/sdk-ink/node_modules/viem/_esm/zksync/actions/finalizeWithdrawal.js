import { parseAbi } from 'abitype';
import { readContract } from '../../actions/public/readContract.js';
import { sendTransaction, } from '../../actions/wallet/sendTransaction.js';
import { AccountNotFoundError } from '../../errors/account.js';
import { ChainNotFoundError, } from '../../errors/chain.js';
import { decodeAbiParameters, encodeFunctionData, parseAccount, slice, } from '../../utils/index.js';
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
    const finalizeWithdrawalParams = await getFinalizeWithdrawalParams(l2Client, {
        hash,
        index,
    });
    const l1Nullifier = (await getBridgeAddresses(client, l2Client)).l1Nullifier;
    const data = encodeFunctionData({
        abi: parseAbi([
            'function finalizeDeposit((uint256 chainId, uint256 l2BatchNumber, uint256 l2MessageIndex, address l2Sender, uint16 l2TxNumberInBatch, bytes message, bytes32[] merkleProof) _finalizeWithdrawalParams)',
        ]),
        functionName: 'finalizeDeposit',
        args: [
            {
                chainId: BigInt(l2Client.chain.id),
                l2BatchNumber: finalizeWithdrawalParams.l1BatchNumber,
                l2MessageIndex: BigInt(finalizeWithdrawalParams.l2MessageIndex),
                l2Sender: finalizeWithdrawalParams.sender,
                l2TxNumberInBatch: Number(finalizeWithdrawalParams.l2TxNumberInBlock),
                message: finalizeWithdrawalParams.message,
                merkleProof: finalizeWithdrawalParams.proof,
            },
        ],
    });
    return await sendTransaction(client, {
        account,
        to: l1Nullifier,
        data,
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
async function getBridgeAddresses(client, l2Client) {
    const addresses = await getDefaultBridgeAddresses(l2Client);
    let l1Nullifier = addresses.l1Nullifier;
    let l1NativeTokenVault = addresses.l1NativeTokenVault;
    if (!l1Nullifier)
        l1Nullifier = await readContract(client, {
            address: addresses.sharedL1,
            abi: parseAbi(['function L1_NULLIFIER() view returns (address)']),
            functionName: 'L1_NULLIFIER',
            args: [],
        });
    if (!l1NativeTokenVault)
        l1NativeTokenVault = await readContract(client, {
            address: addresses.sharedL1,
            abi: parseAbi(['function nativeTokenVault() view returns (address)']),
            functionName: 'nativeTokenVault',
            args: [],
        });
    return {
        ...addresses,
        l1Nullifier,
        l1NativeTokenVault,
    };
}
//# sourceMappingURL=finalizeWithdrawal.js.map