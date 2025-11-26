"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.supportsExecutionMode = supportsExecutionMode;
const readContract_js_1 = require("../../../actions/public/readContract.js");
const withCache_js_1 = require("../../../utils/promise/withCache.js");
const constants_js_1 = require("../constants.js");
const toSerializedMode = {
    default: constants_js_1.executionMode.default,
    opData: constants_js_1.executionMode.opData,
    batchOfBatches: constants_js_1.executionMode.batchOfBatches,
};
async function supportsExecutionMode(client, parameters) {
    const { address, mode: m = 'default' } = parameters;
    const mode = m.startsWith('0x') ? m : toSerializedMode[m];
    try {
        return await (0, withCache_js_1.withCache)(() => (0, readContract_js_1.readContract)(client, {
            abi: constants_js_1.abi,
            address,
            functionName: 'supportsExecutionMode',
            args: [mode],
        }), {
            cacheKey: `supportsExecutionMode.${address}.${mode}`,
        });
    }
    catch {
        return false;
    }
}
//# sourceMappingURL=supportsExecutionMode.js.map