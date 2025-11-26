"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jsonDecrypt = jsonDecrypt;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../base64/index.js");
const decryptData_js_1 = require("./decryptData.js");
function jsonDecrypt({ encoded, encoding }, passphrase) {
    if (!encoded) {
        throw new Error('No encrypted data available to decode');
    }
    return (0, decryptData_js_1.jsonDecryptData)((0, util_1.isHex)(encoded)
        ? (0, util_1.hexToU8a)(encoded)
        : (0, index_js_1.base64Decode)(encoded), passphrase, Array.isArray(encoding.type)
        ? encoding.type
        : [encoding.type]);
}
