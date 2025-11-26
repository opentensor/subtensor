"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.finalizeWithdrawal = finalizeWithdrawal;
const readContract_js_1 = require("../../actions/public/readContract.js");
const sendTransaction_js_1 = require("../../actions/wallet/sendTransaction.js");
const account_js_1 = require("../../errors/account.js");
const chain_js_1 = require("../../errors/chain.js");
const index_js_1 = require("../../utils/index.js");
const abis_js_1 = require("../constants/abis.js");
const address_js_1 = require("../constants/address.js");
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
    const { l1BatchNumber, l2MessageIndex, l2TxNumberInBlock, message, sender, proof, } = await getFinalizeWithdrawalParams(l2Client, { hash, index });
    let l1Bridge;
    if ((0, index_js_1.isAddressEqual)(sender, address_js_1.l2BaseTokenAddress))
        l1Bridge = (await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(l2Client)).sharedL1;
    else if (!(await isLegacyBridge(l2Client, { address: sender })))
        l1Bridge = await (0, readContract_js_1.readContract)(l2Client, {
            address: sender,
            abi: abis_js_1.l2SharedBridgeAbi,
            functionName: 'l1SharedBridge',
            args: [],
        });
    else
        l1Bridge = await (0, readContract_js_1.readContract)(l2Client, {
            address: sender,
            abi: abis_js_1.l2SharedBridgeAbi,
            functionName: 'l1Bridge',
            args: [],
        });
    const data = (0, index_js_1.encodeFunctionData)({
        abi: abis_js_1.l1SharedBridgeAbi,
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
    return await (0, sendTransaction_js_1.sendTransaction)(client, {
        account,
        to: l1Bridge,
        data,
        value: 0n,
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
async function isLegacyBridge(client, parameters) {
    try {
        await (0, readContract_js_1.readContract)(client, {
            address: parameters.address,
            abi: abis_js_1.l2SharedBridgeAbi,
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