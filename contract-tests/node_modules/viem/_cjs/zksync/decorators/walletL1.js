"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.walletActionsL1 = walletActionsL1;
const finalizeWithdrawal_js_1 = require("../actions/finalizeWithdrawal.js");
const requestExecute_js_1 = require("../actions/requestExecute.js");
function walletActionsL1() {
    return (client) => ({
        finalizeWithdrawal: (args) => (0, finalizeWithdrawal_js_1.finalizeWithdrawal)(client, args),
        requestExecute: (args) => (0, requestExecute_js_1.requestExecute)(client, args),
    });
}
//# sourceMappingURL=walletL1.js.map