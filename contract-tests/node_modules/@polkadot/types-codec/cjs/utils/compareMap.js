"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compareMap = compareMap;
const util_1 = require("@polkadot/util");
const util_js_1 = require("./util.js");
function hasMismatch(a, b) {
    return (0, util_1.isUndefined)(a) || ((0, util_js_1.hasEq)(a)
        ? !a.eq(b)
        : a !== b);
}
function notEntry(value) {
    return !Array.isArray(value) || value.length !== 2;
}
function compareMapArray(a, b) {
    // equal number of entries and each entry in the array should match
    return (a.size === b.length) && !b.some((e) => notEntry(e) ||
        hasMismatch(a.get(e[0]), e[1]));
}
function compareMap(a, b) {
    if (Array.isArray(b)) {
        return compareMapArray(a, b);
    }
    else if (b instanceof Map) {
        return compareMapArray(a, [...b.entries()]);
    }
    else if ((0, util_1.isObject)(b)) {
        return compareMapArray(a, Object.entries(b));
    }
    return false;
}
