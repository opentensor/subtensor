"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.walletActionsL2 = walletActionsL2;
const withdraw_js_1 = require("../actions/withdraw.js");
function walletActionsL2() {
    return (client) => ({
        withdraw: (args) => (0, withdraw_js_1.withdraw)(client, args),
    });
}
//# sourceMappingURL=walletL2.js.map