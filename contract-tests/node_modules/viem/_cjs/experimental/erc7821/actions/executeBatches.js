"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.executeBatches = executeBatches;
const sendTransaction_js_1 = require("../../../actions/wallet/sendTransaction.js");
const withCache_js_1 = require("../../../utils/promise/withCache.js");
const errors_js_1 = require("../errors.js");
const encodeExecuteBatchesData_js_1 = require("../utils/encodeExecuteBatchesData.js");
const getExecuteError_js_1 = require("../utils/getExecuteError.js");
const supportsExecutionMode_js_1 = require("./supportsExecutionMode.js");
async function executeBatches(client, parameters) {
    const { authorizationList, batches } = parameters;
    const address = authorizationList?.[0]?.contractAddress ?? parameters.address;
    const supported = await (0, withCache_js_1.withCache)(() => (0, supportsExecutionMode_js_1.supportsExecutionMode)(client, {
        address,
        mode: 'batchOfBatches',
    }), {
        cacheKey: `supportsExecutionMode.${client.uid}.${address}.batchOfBatches`,
    });
    if (!supported)
        throw new errors_js_1.ExecuteUnsupportedError();
    try {
        return await (0, sendTransaction_js_1.sendTransaction)(client, {
            ...parameters,
            to: parameters.address,
            data: (0, encodeExecuteBatchesData_js_1.encodeExecuteBatchesData)({ batches }),
        });
    }
    catch (e) {
        const calls = batches.flatMap((b) => b.calls);
        throw (0, getExecuteError_js_1.getExecuteError)(e, { calls });
    }
}
//# sourceMappingURL=executeBatches.js.map