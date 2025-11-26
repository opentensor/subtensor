"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xglobal = exports.packageInfo = void 0;
exports.extractGlobal = extractGlobal;
exports.exposeGlobal = exposeGlobal;
var packageInfo_js_1 = require("./packageInfo.js");
Object.defineProperty(exports, "packageInfo", { enumerable: true, get: function () { return packageInfo_js_1.packageInfo; } });
/** @internal Last-resort "this", if it gets here it probably would fail anyway */
function evaluateThis(fn) {
    return fn('return this');
}
/**
 * A cross-environment implementation for globalThis
 */
exports.xglobal = (typeof globalThis !== 'undefined'
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
function extractGlobal(name, fallback) {
    // Not quite sure why this is here - snuck in with TS 4.7.2 with no real idea
    // (as of now) as to why this looks like an "any" when we do cast it to a T
    //
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return typeof exports.xglobal[name] === 'undefined'
        ? fallback
        : exports.xglobal[name];
}
/**
 * Expose a value as a known global, if not already defined
 */
function exposeGlobal(name, fallback) {
    if (typeof exports.xglobal[name] === 'undefined') {
        exports.xglobal[name] = fallback;
    }
}
