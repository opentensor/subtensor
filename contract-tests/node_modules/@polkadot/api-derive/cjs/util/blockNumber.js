"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.unwrapBlockNumber = unwrapBlockNumber;
const util_1 = require("@polkadot/util");
function unwrapBlockNumber(hdr) {
    return (0, util_1.isCompact)(hdr.number)
        ? hdr.number.unwrap()
        : hdr.number;
}
