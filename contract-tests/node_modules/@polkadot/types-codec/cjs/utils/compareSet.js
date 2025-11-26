"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compareSet = compareSet;
const util_1 = require("@polkadot/util");
function compareSetArray(a, b) {
    // equal number of entries and each entry in the array should match
    return (a.size === b.length) && !b.some((e) => !a.has(e));
}
function compareSet(a, b) {
    if (Array.isArray(b)) {
        return compareSetArray(a, b);
    }
    else if (b instanceof Set) {
        return compareSetArray(a, [...b.values()]);
    }
    else if ((0, util_1.isObject)(b)) {
        return compareSetArray(a, Object.values(b));
    }
    return false;
}
