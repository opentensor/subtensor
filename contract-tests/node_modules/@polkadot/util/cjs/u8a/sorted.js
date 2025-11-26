"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u8aSorted = u8aSorted;
const cmp_js_1 = require("./cmp.js");
/**
 * @name u8aSorted
 * @summary Sorts an array of Uint8Arrays
 * @description
 * For input `UInt8Array[]` return the sorted result
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aSorted} from '@polkadot/util';
 *
 * u8aSorted([new Uint8Array([0x69]), new Uint8Array([0x68])]); // [0x68, 0x69]
 * ```
 */
function u8aSorted(u8as) {
    return u8as.sort(cmp_js_1.u8aCmp);
}
