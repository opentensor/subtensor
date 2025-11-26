"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeMultiAddress = encodeMultiAddress;
const encode_js_1 = require("./encode.js");
const keyMulti_js_1 = require("./keyMulti.js");
/**
 * @name encodeMultiAddress
 * @summary Creates a multisig address.
 * @description
 * Creates a Substrate multisig address based on the input address and the required threshold.
 */
function encodeMultiAddress(who, threshold, ss58Format) {
    return (0, encode_js_1.encodeAddress)((0, keyMulti_js_1.createKeyMulti)(who, threshold), ss58Format);
}
