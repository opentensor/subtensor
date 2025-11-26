"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isEthereumAddress = isEthereumAddress;
const util_1 = require("@polkadot/util");
const isChecksum_js_1 = require("./isChecksum.js");
function isEthereumAddress(address) {
    if (!address || address.length !== 42 || !(0, util_1.isHex)(address)) {
        return false;
    }
    else if (/^(0x)?[0-9a-f]{40}$/.test(address) || /^(0x)?[0-9A-F]{40}$/.test(address)) {
        return true;
    }
    return (0, isChecksum_js_1.isEthereumChecksum)(address);
}
