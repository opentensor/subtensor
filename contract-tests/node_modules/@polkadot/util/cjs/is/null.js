"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isNull = isNull;
/**
 * @name isNull
 * @summary Tests for a `null` values.
 * @description
 * Checks to see if the input value is `null`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { isNull } from '@polkadot/util';
 *
 * console.log('isNull', isNull(null)); // => true
 * ```
 */
function isNull(value) {
    return value === null;
}
