"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ethereumEncode = ethereumEncode;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../keccak/index.js");
const index_js_2 = require("../secp256k1/index.js");
function getH160(u8a) {
    if ([33, 65].includes(u8a.length)) {
        u8a = (0, index_js_1.keccakAsU8a)((0, index_js_2.secp256k1Expand)(u8a));
    }
    return u8a.slice(-20);
}
function ethereumEncode(addressOrPublic) {
    if (!addressOrPublic) {
        return '0x';
    }
    const u8aAddress = (0, util_1.u8aToU8a)(addressOrPublic);
    if (![20, 32, 33, 65].includes(u8aAddress.length)) {
        throw new Error(`Invalid address or publicKey provided, received ${u8aAddress.length} bytes input`);
    }
    const address = (0, util_1.u8aToHex)(getH160(u8aAddress), -1, false);
    const hash = (0, util_1.u8aToHex)((0, index_js_1.keccakAsU8a)(address), -1, false);
    let result = '';
    for (let i = 0; i < 40; i++) {
        result = `${result}${parseInt(hash[i], 16) > 7 ? address[i].toUpperCase() : address[i]}`;
    }
    return `0x${result}`;
}
