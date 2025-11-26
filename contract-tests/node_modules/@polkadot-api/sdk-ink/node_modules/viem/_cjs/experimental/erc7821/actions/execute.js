"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.execute = execute;
const sendTransaction_js_1 = require("../../../actions/wallet/sendTransaction.js");
const withCache_js_1 = require("../../../utils/promise/withCache.js");
const constants_js_1 = require("../constants.js");
const errors_js_1 = require("../errors.js");
const encodeExecuteData_js_1 = require("../utils/encodeExecuteData.js");
const getExecuteError_js_1 = require("../utils/getExecuteError.js");
const supportsExecutionMode_js_1 = require("./supportsExecutionMode.js");
async function execute(client, parameters) {
    const { authorizationList, calls, opData } = parameters;
    const address = authorizationList?.[0]?.address ?? parameters.address;
    const mode = opData ? constants_js_1.executionMode.opData : constants_js_1.executionMode.default;
    const supported = await (0, withCache_js_1.withCache)(() => (0, supportsExecutionMode_js_1.supportsExecutionMode)(client, {
        address,
        mode,
    }), {
        cacheKey: `supportsExecutionMode.${client.uid}.${address}.${mode}`,
    });
    if (!supported)
        throw new errors_js_1.ExecuteUnsupportedError();
    try {
        return await (0, sendTransaction_js_1.sendTransaction)(client, {
            ...parameters,
            to: parameters.address,
            data: (0, encodeExecuteData_js_1.encodeExecuteData)({ calls, opData }),
        });
    }
    catch (e) {
        throw (0, getExecuteError_js_1.getExecuteError)(e, { calls });
    }
}
//# sourceMappingURL=execute.js.map