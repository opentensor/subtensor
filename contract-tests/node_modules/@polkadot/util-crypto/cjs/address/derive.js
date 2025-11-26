"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deriveAddress = deriveAddress;
const index_js_1 = require("../key/index.js");
const index_js_2 = require("../sr25519/index.js");
const decode_js_1 = require("./decode.js");
const encode_js_1 = require("./encode.js");
function filterHard({ isHard }) {
    return isHard;
}
/**
 * @name deriveAddress
 * @summary Creates a sr25519 derived address from the supplied and path.
 * @description
 * Creates a sr25519 derived address based on the input address/publicKey and the uri supplied.
 */
function deriveAddress(who, suri, ss58Format) {
    const { path } = (0, index_js_1.keyExtractPath)(suri);
    if (!path.length || path.every(filterHard)) {
        throw new Error('Expected suri to contain a combination of non-hard paths');
    }
    let publicKey = (0, decode_js_1.decodeAddress)(who);
    for (const { chainCode } of path) {
        publicKey = (0, index_js_2.sr25519DerivePublic)(publicKey, chainCode);
    }
    return (0, encode_js_1.encodeAddress)(publicKey, ss58Format);
}
