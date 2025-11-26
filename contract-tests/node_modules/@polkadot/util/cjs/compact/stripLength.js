"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compactStripLength = compactStripLength;
const fromU8a_js_1 = require("./fromU8a.js");
/**
 * @name compactStripLength
 * @description Removes the length prefix, returning both the total length (including the value + compact encoding) and the decoded value with the correct length
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactStripLength } from '@polkadot/util';
 *
 * console.log(compactStripLength(new Uint8Array([2 << 2, 0xde, 0xad]))); // [2, Uint8Array[0xde, 0xad]]
 * ```
 */
function compactStripLength(input) {
    const [offset, length] = (0, fromU8a_js_1.compactFromU8a)(input);
    const total = offset + length.toNumber();
    return [
        total,
        input.subarray(offset, total)
    ];
}
