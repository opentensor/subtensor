"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.addSubAccount = addSubAccount;
const index_js_1 = require("../../../utils/index.js");
async function addSubAccount(client, parameters) {
    return client.request({
        method: 'wallet_addSubAccount',
        params: [
            {
                account: {
                    ...parameters,
                    ...(parameters.chainId
                        ? { chainId: (0, index_js_1.numberToHex)(parameters.chainId) }
                        : {}),
                },
                version: '1',
            },
        ],
    });
}
//# sourceMappingURL=addSubAccount.js.map