"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.finalizeWithdrawal = finalizeWithdrawal;
const abitype_1 = require("abitype");
const readContract_js_1 = require("../../actions/public/readContract.js");
const sendTransaction_js_1 = require("../../actions/wallet/sendTransaction.js");
const account_js_1 = require("../../errors/account.js");
const chain_js_1 = require("../../errors/chain.js");
const index_js_1 = require("../../utils/index.js");
const bridge_js_1 = require("../errors/bridge.js");
const getWithdrawalL2ToL1Log_js_1 = require("../utils/bridge/getWithdrawalL2ToL1Log.js");
const getWithdrawalLog_js_1 = require("../utils/bridge/getWithdrawalLog.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
const getLogProof_js_1 = require("./getLogProof.js");
async function finalizeWithdrawal(client, parameters) {
    const { account: account_ = client.account, client: l2Client, hash, index = 0, ...rest } = parameters;
    const account = account_ ? (0, index_js_1.parseAccount)(account_) : client.account;
    if (!account)
        throw new account_js_1.AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new chain_js_1.ChainNotFoundError();
    const finalizeWithdrawalParams = await getFinalizeWithdrawalParams(l2Client, {
        hash,
        index,
    });
    const l1Nullifier = (await getBridgeAddresses(client, l2Client)).l1Nullifier;
    const data = (0, index_js_1.encodeFunctionData)({
        abi: (0, abitype_1.parseAbi)([
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
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        account,
        to: l1Nullifier,
        data,
        ...rest,
    });
}
async function getFinalizeWithdrawalParams(client, parameters) {
    const { hash } = parameters;
    const { log, l1BatchTxId } = await (0, getWithdrawalLog_js_1.getWithdrawalLog)(client, parameters);
    const { l2ToL1LogIndex } = await (0, getWithdrawalL2ToL1Log_js_1.getWithdrawalL2ToL1Log)(client, parameters);
    const sender = (0, index_js_1.slice)(log.topics[1], 12);
    const proof = await (0, getLogProof_js_1.getLogProof)(client, {
        txHash: hash,
        index: l2ToL1LogIndex,
    });
    if (!proof) {
        throw new bridge_js_1.WithdrawalLogNotFoundError({ hash });
    }
    const [message] = (0, index_js_1.decodeAbiParameters)([{ type: 'bytes' }], log.data);
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
    const addresses = await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(l2Client);
    let l1Nullifier = addresses.l1Nullifier;
    let l1NativeTokenVault = addresses.l1NativeTokenVault;
    if (!l1Nullifier)
        l1Nullifier = await (0, readContract_js_1.readContract)(client, {
            address: addresses.sharedL1,
            abi: (0, abitype_1.parseAbi)(['function L1_NULLIFIER() view returns (address)']),
            functionName: 'L1_NULLIFIER',
            args: [],
        });
    if (!l1NativeTokenVault)
        l1NativeTokenVault = await (0, readContract_js_1.readContract)(client, {
            address: addresses.sharedL1,
            abi: (0, abitype_1.parseAbi)(['function nativeTokenVault() view returns (address)']),
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