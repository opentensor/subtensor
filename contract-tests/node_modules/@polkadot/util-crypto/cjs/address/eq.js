"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.addressEq = addressEq;
const util_1 = require("@polkadot/util");
const decode_js_1 = require("./decode.js");
/**
 * @name addressEq
 * @summary Compares two addresses, either in ss58, Uint8Array or hex format.
 * @description
 * For the input values, return true is the underlying public keys do match.
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aEq } from '@polkadot/util';
 *
 * u8aEq(new Uint8Array([0x68, 0x65]), new Uint8Array([0x68, 0x65])); // true
 * ```
 */
function addressEq(a, b) {
    return (0, util_1.u8aEq)((0, decode_js_1.decodeAddress)(a), (0, decode_js_1.decodeAddress)(b));
}
