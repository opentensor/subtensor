import { objectSpread } from './spread.js';
/**
 * @name objectCopy
 * @summary Creates a shallow clone of the input object
 */
export function objectCopy(source) {
    return objectSpread({}, source);
}
