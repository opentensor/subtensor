"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.memoize = memoize;
const stringify_js_1 = require("./stringify.js");
function defaultGetId() {
    return 'none';
}
/**
 * @name memoize
 * @description Memomize the function with a specific instanceId
 */
function memoize(fn, { getInstanceId = defaultGetId } = {}) {
    const cache = {};
    const memoized = (...args) => {
        const stringParams = (0, stringify_js_1.stringify)(args);
        const instanceId = getInstanceId();
        if (!cache[instanceId]) {
            cache[instanceId] = {};
        }
        if (cache[instanceId][stringParams] === undefined) {
            cache[instanceId][stringParams] = fn(...args);
        }
        return cache[instanceId][stringParams];
    };
    memoized.unmemoize = (...args) => {
        const stringParams = (0, stringify_js_1.stringify)(args);
        const instanceId = getInstanceId();
        if (cache[instanceId]?.[stringParams] !== undefined) {
            delete cache[instanceId][stringParams];
        }
    };
    return memoized;
}
