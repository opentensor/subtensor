"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getCapabilities = getCapabilities;
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
async function getCapabilities(client, parameters = {}) {
    const { account = client.account, chainId } = parameters;
    const account_ = account ? (0, parseAccount_js_1.parseAccount)(account) : undefined;
    const params = chainId
        ? [account_?.address, [(0, toHex_js_1.numberToHex)(chainId)]]
        : [account_?.address];
    const capabilities_raw = await client.request({
        method: 'wallet_getCapabilities',
        params,
    });
    const capabilities = {};
    for (const [chainId, capabilities_] of Object.entries(capabilities_raw)) {
        capabilities[Number(chainId)] = {};
        for (let [key, value] of Object.entries(capabilities_)) {
            if (key === 'addSubAccount')
                key = 'unstable_addSubAccount';
            capabilities[Number(chainId)][key] = value;
        }
    }
    return (typeof chainId === 'number' ? capabilities[chainId] : capabilities);
}
//# sourceMappingURL=getCapabilities.js.map