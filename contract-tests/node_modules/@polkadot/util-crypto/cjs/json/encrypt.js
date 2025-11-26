"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jsonEncrypt = jsonEncrypt;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../nacl/index.js");
const index_js_2 = require("../scrypt/index.js");
const encryptFormat_js_1 = require("./encryptFormat.js");
function jsonEncrypt(data, contentType, passphrase) {
    let isEncrypted = false;
    let encoded = data;
    if (passphrase) {
        const { params, password, salt } = (0, index_js_2.scryptEncode)(passphrase);
        const { encrypted, nonce } = (0, index_js_1.naclEncrypt)(encoded, password.subarray(0, 32));
        isEncrypted = true;
        encoded = (0, util_1.u8aConcat)((0, index_js_2.scryptToU8a)(salt, params), nonce, encrypted);
    }
    return (0, encryptFormat_js_1.jsonEncryptFormat)(encoded, contentType, isEncrypted);
}
