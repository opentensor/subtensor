"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getWithdrawalL2ToL1Log = getWithdrawalL2ToL1Log;
const index_js_1 = require("../../../actions/index.js");
const index_js_2 = require("../../../utils/index.js");
const address_js_1 = require("../../constants/address.js");
async function getWithdrawalL2ToL1Log(client, parameters) {
    const { hash, index = 0 } = parameters;
    const receipt = (await (0, index_js_1.getTransactionReceipt)(client, {
        hash,
    }));
    const messages = Array.from(receipt.l2ToL1Logs.entries()).filter(([, log]) => (0, index_js_2.isAddressEqual)(log.sender, address_js_1.l1MessengerAddress));
    const [l2ToL1LogIndex, l2ToL1Log] = messages[index];
    return {
        l2ToL1LogIndex,
        l2ToL1Log,
    };
}
//# sourceMappingURL=getWithdrawalL2ToL1Log.js.map