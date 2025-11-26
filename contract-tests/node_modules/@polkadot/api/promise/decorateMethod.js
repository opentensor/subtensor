import { catchError, EMPTY, tap } from 'rxjs';
import { isFunction, nextTick } from '@polkadot/util';
export function promiseTracker(resolve, reject) {
    let isCompleted = false;
    return {
        reject: (error) => {
            if (!isCompleted) {
                isCompleted = true;
                reject(error);
            }
            return EMPTY;
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
    const callback = (args.length && isFunction(args[args.length - 1]))
        ? actualArgs.pop()
        : undefined;
    // When we need a subscription, ensure that a valid callback is actually passed
    if (needsCallback && !isFunction(callback)) {
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
            .pipe(catchError((error) => tracker.reject(error)))
            .subscribe((result) => {
            tracker.resolve(result);
            nextTick(() => subscription.unsubscribe());
        });
    });
}
function decorateSubscribe(method, args, resultCb) {
    return new Promise((resolve, reject) => {
        // either reject with error or resolve with unsubscribe callback
        const tracker = promiseTracker(resolve, reject);
        // errors reject immediately, the first result resolves with an unsubscribe promise, all results via callback
        const subscription = method(...args)
            .pipe(catchError((error) => tracker.reject(error)), tap(() => tracker.resolve(() => subscription.unsubscribe())))
            .subscribe((result) => {
            // queue result (back of queue to clear current)
            nextTick(() => resultCb(result));
        });
    });
}
/**
 * @description Decorate method for ApiPromise, where the results are converted to the Promise equivalent
 */
export function toPromiseMethod(method, options) {
    const needsCallback = !!(options?.methodName && options.methodName.includes('subscribe'));
    return function (...args) {
        const [actualArgs, resultCb] = extractArgs(args, needsCallback);
        return resultCb
            ? decorateSubscribe(method, actualArgs, resultCb)
            : decorateCall(options?.overrideNoSub || method, actualArgs);
    };
}
