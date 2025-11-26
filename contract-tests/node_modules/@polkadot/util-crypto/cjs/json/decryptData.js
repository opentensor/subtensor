"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jsonDecryptData = jsonDecryptData;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../nacl/index.js");
const index_js_2 = require("../scrypt/index.js");
const constants_js_1 = require("./constants.js");
function jsonDecryptData(encrypted, passphrase, encType = constants_js_1.ENCODING) {
    if (!encrypted) {
        throw new Error('No encrypted data available to decode');
    }
    else if (encType.includes('xsalsa20-poly1305') && !passphrase) {
        throw new Error('Password required to decode encrypted data');
    }
    let encoded = encrypted;
    if (passphrase) {
        let password;
        if (encType.includes('scrypt')) {
            const { params, salt } = (0, index_js_2.scryptFromU8a)(encrypted);
            password = (0, index_js_2.scryptEncode)(passphrase, salt, params).password;
            encrypted = encrypted.subarray(constants_js_1.SCRYPT_LENGTH);
        }
        else {
            password = (0, util_1.stringToU8a)(passphrase);
        }
        encoded = (0, index_js_1.naclDecrypt)(encrypted.subarray(constants_js_1.NONCE_LENGTH), encrypted.subarray(0, constants_js_1.NONCE_LENGTH), (0, util_1.u8aFixLength)(password, 256, true));
    }
    if (!encoded) {
        throw new Error('Unable to decode using the supplied passphrase');
    }
    return encoded;
}
