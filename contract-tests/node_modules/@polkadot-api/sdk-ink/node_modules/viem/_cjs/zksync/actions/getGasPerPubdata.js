"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getGasPerPubdata = getGasPerPubdata;
const fromHex_js_1 = require("../../utils/encoding/fromHex.js");
async function getGasPerPubdata(client) {
    const result = await client.request({
        method: 'zks_gasPerPubdata',
    }, {
        dedupe: true,
    });
    return (0, fromHex_js_1.hexToBigInt)(result);
}
//# sourceMappingURL=getGasPerPubdata.js.map