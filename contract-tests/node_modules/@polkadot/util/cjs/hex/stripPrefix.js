"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hexStripPrefix = hexStripPrefix;
const hex_js_1 = require("../is/hex.js");
/**
 * @name hexStripPrefix
 * @summary Strips any leading `0x` prefix.
 * @description
 * Tests for the existence of a `0x` prefix, and returns the value without the prefix. Un-prefixed values are returned as-is.
 * @example
 * <BR>
 *
 * ```javascript
 * import { hexStripPrefix } from '@polkadot/util';
 *
 * console.log('stripped', hexStripPrefix('0x1234')); // => 1234
 * ```
 */
function hexStripPrefix(value) {
    if (!value || value === '0x') {
        return '';
    }
    else if (hex_js_1.REGEX_HEX_PREFIXED.test(value)) {
        return value.substring(2);
    }
    else if (hex_js_1.REGEX_HEX_NOPREFIX.test(value)) {
        return value;
    }
    throw new Error(`Expected hex value to convert, found '${value}'`);
}
