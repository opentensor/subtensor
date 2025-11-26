"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u8aToBuffer = u8aToBuffer;
const x_global_1 = require("@polkadot/x-global");
const has_js_1 = require("../has.js");
/**
 * @name u8aToBuffer
 * @summary Creates a Buffer object from a hex string.
 * @description
 * `null` inputs returns an empty `Buffer` result. `UInt8Array` input values return the actual bytes value converted to a `Buffer`. Anything that is not a `UInt8Array` throws an error.
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aToBuffer } from '@polkadot/util';
 *
 * console.log('Buffer', u8aToBuffer(new Uint8Array([1, 2, 3])));
 * ```
 */
function u8aToBuffer(value) {
    return has_js_1.hasBuffer
        ? x_global_1.xglobal.Buffer.from(value || [])
        : new Uint8Array(value || []);
}
