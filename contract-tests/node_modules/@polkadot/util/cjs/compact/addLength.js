"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.compactAddLength = compactAddLength;
const index_js_1 = require("../u8a/index.js");
const toU8a_js_1 = require("./toU8a.js");
/**
 * @name compactAddLength
 * @description Adds a length prefix to the input value
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactAddLength } from '@polkadot/util';
 *
 * console.log(compactAddLength(new Uint8Array([0xde, 0xad, 0xbe, 0xef]))); // Uint8Array([4 << 2, 0xde, 0xad, 0xbe, 0xef])
 * ```
 */
function compactAddLength(input) {
    return (0, index_js_1.u8aConcatStrict)([
        (0, toU8a_js_1.compactToU8a)(input.length),
        input
    ]);
}
