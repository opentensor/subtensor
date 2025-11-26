"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.writeContractSync = writeContractSync;
const sendTransactionSync_js_1 = require("./sendTransactionSync.js");
const writeContract_js_1 = require("./writeContract.js");
async function writeContractSync(client, parameters) {
    return writeContract_js_1.writeContract.internal(client, sendTransactionSync_js_1.sendTransactionSync, 'sendTransactionSync', parameters);
}
//# sourceMappingURL=writeContractSync.js.map