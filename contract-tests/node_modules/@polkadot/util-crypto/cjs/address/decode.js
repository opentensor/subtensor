"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decodeAddress = decodeAddress;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../base58/index.js");
const checksum_js_1 = require("./checksum.js");
const defaults_js_1 = require("./defaults.js");
function decodeAddress(encoded, ignoreChecksum, ss58Format = -1) {
    if (!encoded) {
        throw new Error('Invalid empty address passed');
    }
    if ((0, util_1.isU8a)(encoded) || (0, util_1.isHex)(encoded)) {
        return (0, util_1.u8aToU8a)(encoded);
    }
    try {
        const decoded = (0, index_js_1.base58Decode)(encoded);
        if (!defaults_js_1.defaults.allowedEncodedLengths.includes(decoded.length)) {
            throw new Error('Invalid decoded address length');
        }
        const [isValid, endPos, ss58Length, ss58Decoded] = (0, checksum_js_1.checkAddressChecksum)(decoded);
        if (!isValid && !ignoreChecksum) {
            throw new Error('Invalid decoded address checksum');
        }
        else if (ss58Format !== -1 && ss58Format !== ss58Decoded) {
            throw new Error(`Expected ss58Format ${ss58Format}, received ${ss58Decoded}`);
        }
        return decoded.slice(ss58Length, endPos);
    }
    catch (error) {
        throw new Error(`Decoding ${encoded}: ${error.message}`);
    }
}
