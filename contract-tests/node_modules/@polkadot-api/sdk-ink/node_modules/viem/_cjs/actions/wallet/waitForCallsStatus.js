"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WaitForCallsStatusTimeoutError = void 0;
exports.waitForCallsStatus = waitForCallsStatus;
const base_js_1 = require("../../errors/base.js");
const calls_js_1 = require("../../errors/calls.js");
const getAction_js_1 = require("../../utils/getAction.js");
const observe_js_1 = require("../../utils/observe.js");
const poll_js_1 = require("../../utils/poll.js");
const withResolvers_js_1 = require("../../utils/promise/withResolvers.js");
const withRetry_js_1 = require("../../utils/promise/withRetry.js");
const stringify_js_1 = require("../../utils/stringify.js");
const getCallsStatus_js_1 = require("./getCallsStatus.js");
async function waitForCallsStatus(client, parameters) {
    const { id, pollingInterval = client.pollingInterval, status = ({ statusCode }) => statusCode === 200 || statusCode >= 300, retryCount = 4, retryDelay = ({ count }) => ~~(1 << count) * 200, timeout = 60_000, throwOnFailure = false, } = parameters;
    const observerId = (0, stringify_js_1.stringify)(['waitForCallsStatus', client.uid, id]);
    const { promise, resolve, reject } = (0, withResolvers_js_1.withResolvers)();
    let timer;
    const unobserve = (0, observe_js_1.observe)(observerId, { resolve, reject }, (emit) => {
        const unpoll = (0, poll_js_1.poll)(async () => {
            const done = (fn) => {
                clearTimeout(timer);
                unpoll();
                fn();
                unobserve();
            };
            try {
                const result = await (0, withRetry_js_1.withRetry)(async () => {
                    const result = await (0, getAction_js_1.getAction)(client, getCallsStatus_js_1.getCallsStatus, 'getCallsStatus')({ id });
                    if (throwOnFailure && result.status === 'failure')
                        throw new calls_js_1.BundleFailedError(result);
                    return result;
                }, {
                    retryCount,
                    delay: retryDelay,
                });
                if (!status(result))
                    return;
                done(() => emit.resolve(result));
            }
            catch (error) {
                done(() => emit.reject(error));
            }
        }, {
            interval: pollingInterval,
            emitOnBegin: true,
        });
        return unpoll;
    });
    timer = timeout
        ? setTimeout(() => {
            unobserve();
            clearTimeout(timer);
            reject(new WaitForCallsStatusTimeoutError({ id }));
        }, timeout)
        : undefined;
    return await promise;
}
class WaitForCallsStatusTimeoutError extends base_js_1.BaseError {
    constructor({ id }) {
        super(`Timed out while waiting for call bundle with id "${id}" to be confirmed.`, { name: 'WaitForCallsStatusTimeoutError' });
    }
}
exports.WaitForCallsStatusTimeoutError = WaitForCallsStatusTimeoutError;
//# sourceMappingURL=waitForCallsStatus.js.map