"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hexHasPrefix = hexHasPrefix;
const hex_js_1 = require("../is/hex.js");
/**
 * @name hexHasPrefix
 * @summary Tests for the existence of a `0x` prefix.
 * @description
 * Checks for a valid hex input value and if the start matched `0x`
 * @example
 * <BR>
 *
 * ```javascript
 * import { hexHasPrefix } from '@polkadot/util';
 *
 * console.log('has prefix', hexHasPrefix('0x1234')); // => true
 * ```
 */
function hexHasPrefix(value) {
    return !!value && (0, hex_js_1.isHex)(value, -1);
}
