"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.promiseTracker = promiseTracker;
exports.toPromiseMethod = toPromiseMethod;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
function promiseTracker(resolve, reject) {
    let isCompleted = false;
    return {
        reject: (error) => {
            if (!isCompleted) {
                isCompleted = true;
                reject(error);
            }
            return rxjs_1.EMPTY;
        },
        resolve: (value) => {
            if (!isCompleted) {
                isCompleted = true;
                resolve(value);
            }
        }
    };
}
function extractArgs(args, needsCallback) {
    const actualArgs = args.slice();
    // If the last arg is a function, we pop it, put it into callback.
    // actualArgs will then hold the actual arguments to be passed to `method`
    const callback = (args.length && (0, util_1.isFunction)(args[args.length - 1]))
        ? actualArgs.pop()
        : undefined;
    // When we need a subscription, ensure that a valid callback is actually passed
    if (needsCallback && !(0, util_1.isFunction)(callback)) {
        throw new Error('Expected a callback to be passed with subscriptions');
    }
    return [actualArgs, callback];
}
function decorateCall(method, args) {
    return new Promise((resolve, reject) => {
        // single result tracker - either reject with Error or resolve with Codec result
        const tracker = promiseTracker(resolve, reject);
        // encoding errors reject immediately, any result unsubscribes and resolves
        const subscription = method(...args)
            .pipe((0, rxjs_1.catchError)((error) => tracker.reject(error)))
            .subscribe((result) => {
            tracker.resolve(result);
            (0, util_1.nextTick)(() => subscription.unsubscribe());
        });
    });
}
function decorateSubscribe(method, args, resultCb) {
    return new Promise((resolve, reject) => {
        // either reject with error or resolve with unsubscribe callback
        const tracker = promiseTracker(resolve, reject);
        // errors reject immediately, the first result resolves with an unsubscribe promise, all results via callback
        const subscription = method(...args)
            .pipe((0, rxjs_1.catchError)((error) => tracker.reject(error)), (0, rxjs_1.tap)(() => tracker.resolve(() => subscription.unsubscribe())))
            .subscribe((result) => {
            // queue result (back of queue to clear current)
            (0, util_1.nextTick)(() => resultCb(result));
        });
    });
}
/**
 * @description Decorate method for ApiPromise, where the results are converted to the Promise equivalent
 */
function toPromiseMethod(method, options) {
    const needsCallback = !!(options?.methodName && options.methodName.includes('subscribe'));
    return function (...args) {
        const [actualArgs, resultCb] = extractArgs(args, needsCallback);
        return resultCb
            ? decorateSubscribe(method, actualArgs, resultCb)
            : decorateCall(options?.overrideNoSub || method, actualArgs);
    };
}
