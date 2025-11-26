"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.RpcCoder = void 0;
const tslib_1 = require("tslib");
const util_1 = require("@polkadot/util");
const error_js_1 = tslib_1.__importDefault(require("./error.js"));
function formatErrorData(data) {
    if ((0, util_1.isUndefined)(data)) {
        return '';
    }
    const formatted = `: ${(0, util_1.isString)(data)
        ? data.replace(/Error\("/g, '').replace(/\("/g, '(').replace(/"\)/g, ')').replace(/\(/g, ', ').replace(/\)/g, '')
        : (0, util_1.stringify)(data)}`;
    // We need some sort of cut-off here since these can be very large and
    // very nested, pick a number and trim the result display to it
    return formatted.length <= 256
        ? formatted
        : `${formatted.substring(0, 255)}…`;
}
function checkError(error) {
    if (error) {
        const { code, data, message } = error;
        throw new error_js_1.default(`${code}: ${message}${formatErrorData(data)}`, code, data);
    }
}
/** @internal */
class RpcCoder {
    #id = 0;
    decodeResponse(response) {
        if (!response || response.jsonrpc !== '2.0') {
            throw new Error('Invalid jsonrpc field in decoded object');
        }
        const isSubscription = !(0, util_1.isUndefined)(response.params) && !(0, util_1.isUndefined)(response.method);
        if (!(0, util_1.isNumber)(response.id) &&
            (!isSubscription || (!(0, util_1.isNumber)(response.params.subscription) &&
                !(0, util_1.isString)(response.params.subscription)))) {
            throw new Error('Invalid id field in decoded object');
        }
        checkError(response.error);
        if (response.result === undefined && !isSubscription) {
            throw new Error('No result found in jsonrpc response');
        }
        if (isSubscription) {
            checkError(response.params.error);
            return response.params.result;
        }
        return response.result;
    }
    encodeJson(method, params) {
        const [id, data] = this.encodeObject(method, params);
        return [id, (0, util_1.stringify)(data)];
    }
    encodeObject(method, params) {
        const id = ++this.#id;
        return [id, {
                id,
                jsonrpc: '2.0',
                method,
                params
            }];
    }
}
exports.RpcCoder = RpcCoder;
