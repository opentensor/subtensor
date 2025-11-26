"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isWithdrawalFinalized = isWithdrawalFinalized;
const readContract_js_1 = require("../../actions/public/readContract.js");
const chain_js_1 = require("../../errors/chain.js");
const abis_js_1 = require("../constants/abis.js");
const bridge_js_1 = require("../errors/bridge.js");
const getWithdrawalL2ToL1Log_js_1 = require("../utils/bridge/getWithdrawalL2ToL1Log.js");
const getWithdrawalLog_js_1 = require("../utils/bridge/getWithdrawalLog.js");
const getDefaultBridgeAddresses_js_1 = require("./getDefaultBridgeAddresses.js");
const getLogProof_js_1 = require("./getLogProof.js");
async function isWithdrawalFinalized(client, parameters) {
    const { client: l2Client, hash, index = 0 } = parameters;
    if (!l2Client.chain)
        throw new chain_js_1.ChainNotFoundError();
    const { log } = await (0, getWithdrawalLog_js_1.getWithdrawalLog)(l2Client, { hash, index });
    const { l2ToL1LogIndex } = await (0, getWithdrawalL2ToL1Log_js_1.getWithdrawalL2ToL1Log)(l2Client, {
        hash,
        index,
    });
    const proof = await (0, getLogProof_js_1.getLogProof)(l2Client, {
        txHash: hash,
        index: l2ToL1LogIndex,
    });
    if (!proof)
        throw new bridge_js_1.WithdrawalLogNotFoundError({ hash });
    const l1Bridge = (await (0, getDefaultBridgeAddresses_js_1.getDefaultBridgeAddresses)(l2Client)).sharedL1;
    return await (0, readContract_js_1.readContract)(client, {
        address: l1Bridge,
        abi: abis_js_1.l1SharedBridgeAbi,
        functionName: 'isWithdrawalFinalized',
        args: [BigInt(l2Client.chain.id), log.l1BatchNumber, BigInt(proof.id)],
    });
}
//# sourceMappingURL=isWithdrawalFinalized.js.map