import { isNumber, isString, isUndefined, stringify } from '@polkadot/util';
import RpcError from './error.js';
function formatErrorData(data) {
    if (isUndefined(data)) {
        return '';
    }
    const formatted = `: ${isString(data)
        ? data.replace(/Error\("/g, '').replace(/\("/g, '(').replace(/"\)/g, ')').replace(/\(/g, ', ').replace(/\)/g, '')
        : stringify(data)}`;
    // We need some sort of cut-off here since these can be very large and
    // very nested, pick a number and trim the result display to it
    return formatted.length <= 256
        ? formatted
        : `${formatted.substring(0, 255)}…`;
}
function checkError(error) {
    if (error) {
        const { code, data, message } = error;
        throw new RpcError(`${code}: ${message}${formatErrorData(data)}`, code, data);
    }
}
/** @internal */
export class RpcCoder {
    #id = 0;
    decodeResponse(response) {
        if (!response || response.jsonrpc !== '2.0') {
            throw new Error('Invalid jsonrpc field in decoded object');
        }
        const isSubscription = !isUndefined(response.params) && !isUndefined(response.method);
        if (!isNumber(response.id) &&
            (!isSubscription || (!isNumber(response.params.subscription) &&
                !isString(response.params.subscription)))) {
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
        return [id, stringify(data)];
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
