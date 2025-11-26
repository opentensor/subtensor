export { packageInfo } from './packageInfo.js';
/** @internal Last-resort "this", if it gets here it probably would fail anyway */
function evaluateThis(fn) {
    return fn('return this');
}
/**
 * A cross-environment implementation for globalThis
 */
export const xglobal = /*#__PURE__*/ (typeof globalThis !== 'undefined'
    ? globalThis
    : typeof global !== 'undefined'
        ? global
        : typeof self !== 'undefined'
            ? self
            : typeof window !== 'undefined'
                ? window
                : evaluateThis(Function));
/**
 * Extracts a known global from the environment, applying a fallback if not found
 */
export function extractGlobal(name, fallback) {
    // Not quite sure why this is here - snuck in with TS 4.7.2 with no real idea
    // (as of now) as to why this looks like an "any" when we do cast it to a T
    //
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return typeof xglobal[name] === 'undefined'
        ? fallback
        : xglobal[name];
}
/**
 * Expose a value as a known global, if not already defined
 */
export function exposeGlobal(name, fallback) {
    if (typeof xglobal[name] === 'undefined') {
        xglobal[name] = fallback;
    }
}
