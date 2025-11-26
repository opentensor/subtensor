"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hashAuthorization = hashAuthorization;
const concat_js_1 = require("../data/concat.js");
const toBytes_js_1 = require("../encoding/toBytes.js");
const toHex_js_1 = require("../encoding/toHex.js");
const toRlp_js_1 = require("../encoding/toRlp.js");
const keccak256_js_1 = require("../hash/keccak256.js");
function hashAuthorization(parameters) {
    const { chainId, nonce, to } = parameters;
    const address = parameters.contractAddress ?? parameters.address;
    const hash = (0, keccak256_js_1.keccak256)((0, concat_js_1.concatHex)([
        '0x05',
        (0, toRlp_js_1.toRlp)([
            chainId ? (0, toHex_js_1.numberToHex)(chainId) : '0x',
            address,
            nonce ? (0, toHex_js_1.numberToHex)(nonce) : '0x',
        ]),
    ]));
    if (to === 'bytes')
        return (0, toBytes_js_1.hexToBytes)(hash);
    return hash;
}
//# sourceMappingURL=hashAuthorization.js.map