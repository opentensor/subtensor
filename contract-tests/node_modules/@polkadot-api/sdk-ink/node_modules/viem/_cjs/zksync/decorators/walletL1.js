"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.walletActionsL1 = walletActionsL1;
const claimFailedDeposit_js_1 = require("../actions/claimFailedDeposit.js");
const deposit_js_1 = require("../actions/deposit.js");
const finalizeWithdrawal_js_1 = require("../actions/finalizeWithdrawal.js");
const requestExecute_js_1 = require("../actions/requestExecute.js");
function walletActionsL1() {
    return (client) => ({
        claimFailedDeposit: (args) => (0, claimFailedDeposit_js_1.claimFailedDeposit)(client, args),
        deposit: (args) => (0, deposit_js_1.deposit)(client, args),
        finalizeWithdrawal: (args) => (0, finalizeWithdrawal_js_1.finalizeWithdrawal)(client, args),
        requestExecute: (args) => (0, requestExecute_js_1.requestExecute)(client, args),
    });
}
//# sourceMappingURL=walletL1.js.map