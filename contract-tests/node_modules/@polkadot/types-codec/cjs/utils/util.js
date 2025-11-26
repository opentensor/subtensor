"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hasEq = hasEq;
const util_1 = require("@polkadot/util");
function hasEq(o) {
    return (0, util_1.isFunction)(o.eq);
}
