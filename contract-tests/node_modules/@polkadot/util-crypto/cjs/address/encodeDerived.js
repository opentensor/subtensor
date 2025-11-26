"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeDerivedAddress = encodeDerivedAddress;
const decode_js_1 = require("./decode.js");
const encode_js_1 = require("./encode.js");
const keyDerived_js_1 = require("./keyDerived.js");
/**
 * @name encodeDerivedAddress
 * @summary Creates a derived address as used in Substrate utility.
 * @description
 * Creates a Substrate derived address based on the input address/publicKey and the index supplied.
 */
function encodeDerivedAddress(who, index, ss58Format) {
    return (0, encode_js_1.encodeAddress)((0, keyDerived_js_1.createKeyDerived)((0, decode_js_1.decodeAddress)(who), index), ss58Format);
}
