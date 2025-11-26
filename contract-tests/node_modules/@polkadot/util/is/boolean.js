/**
 * @name isBoolean
 * @summary Tests for a boolean value.
 * @description
 * Checks to see if the input value is a JavaScript boolean.
 * @example
 * <BR>
 *
 * ```javascript
 * import { isBoolean } from '@polkadot/util';
 *
 * isBoolean(false); // => true
 * ```
 */
export function isBoolean(value) {
    return typeof value === 'boolean';
}
