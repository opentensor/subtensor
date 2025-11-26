import { createCmp } from './helpers.js';
/**
 * @name nMax
 * @summary Finds and returns the highest value in an array of bigint.
 */
export const nMax = /*#__PURE__*/ createCmp((a, b) => a > b);
/**
 * @name nMin
 * @summary Finds and returns the lowest value in an array of bigint.
 */
export const nMin = /*#__PURE__*/ createCmp((a, b) => a < b);
