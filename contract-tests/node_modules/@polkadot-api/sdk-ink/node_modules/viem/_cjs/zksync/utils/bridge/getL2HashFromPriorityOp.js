"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getL2HashFromPriorityOp = getL2HashFromPriorityOp;
const index_js_1 = require("../../../utils/index.js");
const abis_js_1 = require("../../constants/abis.js");
const bridge_js_1 = require("../../errors/bridge.js");
function getL2HashFromPriorityOp(receipt, zksync) {
    let hash = null;
    for (const log of receipt.logs) {
        if (!(0, index_js_1.isAddressEqual)(log.address, zksync))
            continue;
        try {
            const priorityQueueLog = (0, index_js_1.decodeEventLog)({
                abi: abis_js_1.zksyncAbi,
                data: log.data,
                topics: log.topics,
            });
            if (priorityQueueLog && priorityQueueLog.args.txHash !== null)
                hash = priorityQueueLog.args.txHash;
        }
        catch (_e) { }
    }
    if (!hash) {
        throw new bridge_js_1.TxHashNotFoundInLogsError();
    }
    return hash;
}
//# sourceMappingURL=getL2HashFromPriorityOp.js.map