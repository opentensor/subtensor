"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.checkAddress = checkAddress;
const index_js_1 = require("../base58/index.js");
const checksum_js_1 = require("./checksum.js");
const defaults_js_1 = require("./defaults.js");
/**
 * @name checkAddress
 * @summary Validates an ss58 address.
 * @description
 * From the provided input, validate that the address is a valid input.
 */
function checkAddress(address, prefix) {
    let decoded;
    try {
        decoded = (0, index_js_1.base58Decode)(address);
    }
    catch (error) {
        return [false, error.message];
    }
    const [isValid, , , ss58Decoded] = (0, checksum_js_1.checkAddressChecksum)(decoded);
    if (ss58Decoded !== prefix) {
        return [false, `Prefix mismatch, expected ${prefix}, found ${ss58Decoded}`];
    }
    else if (!defaults_js_1.defaults.allowedEncodedLengths.includes(decoded.length)) {
        return [false, 'Invalid decoded address length'];
    }
    return [isValid, isValid ? null : 'Invalid decoded address checksum'];
}
