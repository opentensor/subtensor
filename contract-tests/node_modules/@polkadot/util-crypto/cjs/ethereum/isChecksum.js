"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isEthereumChecksum = isEthereumChecksum;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../keccak/index.js");
function isInvalidChar(char, byte) {
    return char !== (byte > 7
        ? char.toUpperCase()
        : char.toLowerCase());
}
function isEthereumChecksum(_address) {
    const address = _address.replace('0x', '');
    const hash = (0, util_1.u8aToHex)((0, index_js_1.keccakAsU8a)(address.toLowerCase()), -1, false);
    for (let i = 0; i < 40; i++) {
        if (isInvalidChar(address[i], parseInt(hash[i], 16))) {
            return false;
        }
    }
    return true;
}
