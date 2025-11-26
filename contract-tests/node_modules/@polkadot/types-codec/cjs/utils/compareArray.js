"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compareArray = compareArray;
const util_1 = require("@polkadot/util");
const util_js_1 = require("./util.js");
function compareArray(a, b) {
    if (Array.isArray(b)) {
        return (a.length === b.length) && (0, util_1.isUndefined)(a.find((v, index) => (0, util_js_1.hasEq)(v)
            ? !v.eq(b[index])
            : v !== b[index]));
    }
    return false;
}
