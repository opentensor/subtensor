"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BN_SQRT_MAX_INTEGER = exports.BN_MAX_INTEGER = exports.BN_QUINTILL = exports.BN_BILLION = exports.BN_MILLION = exports.BN_THOUSAND = exports.BN_HUNDRED = exports.BN_TEN = exports.BN_NINE = exports.BN_EIGHT = exports.BN_SEVEN = exports.BN_SIX = exports.BN_FIVE = exports.BN_FOUR = exports.BN_THREE = exports.BN_TWO = exports.BN_ONE = exports.BN_ZERO = void 0;
const bn_js_1 = require("./bn.js");
/**
 * @name BN_ZERO
 * @summary BN constant for 0.
 */
exports.BN_ZERO = new bn_js_1.BN(0);
/**
 * @name BN_ONE
 * @summary BN constant for 1.
 */
exports.BN_ONE = new bn_js_1.BN(1);
/**
 * @name BN_TWO
 * @summary BN constant for 2.
 */
exports.BN_TWO = new bn_js_1.BN(2);
/**
 * @name BN_THREE
 * @summary BN constant for 3.
 */
exports.BN_THREE = new bn_js_1.BN(3);
/**
 * @name BN_FOUR
 * @summary BN constant for 4.
 */
exports.BN_FOUR = new bn_js_1.BN(4);
/**
 * @name BN_FIVE
 * @summary BN constant for 5.
 */
exports.BN_FIVE = new bn_js_1.BN(5);
/**
 * @name BN_SIX
 * @summary BN constant for 6.
 */
exports.BN_SIX = new bn_js_1.BN(6);
/**
 * @name BN_SEVEN
 * @summary BN constant for 7.
 */
exports.BN_SEVEN = new bn_js_1.BN(7);
/**
 * @name BN_EIGHT
 * @summary BN constant for 8.
 */
exports.BN_EIGHT = new bn_js_1.BN(8);
/**
 * @name BN_NINE
 * @summary BN constant for 9.
 */
exports.BN_NINE = new bn_js_1.BN(9);
/**
 * @name BN_TEN
 * @summary BN constant for 10.
 */
exports.BN_TEN = new bn_js_1.BN(10);
/**
 * @name BN_HUNDRED
 * @summary BN constant for 100.
 */
exports.BN_HUNDRED = new bn_js_1.BN(100);
/**
 * @name BN_THOUSAND
 * @summary BN constant for 1,000.
 */
exports.BN_THOUSAND = new bn_js_1.BN(1_000);
/**
 * @name BN_MILLION
 * @summary BN constant for 1,000,000.
 */
exports.BN_MILLION = new bn_js_1.BN(1_000_000);
/**
 * @name BN_BILLION
 * @summary BN constant for 1,000,000,000.
 */
exports.BN_BILLION = new bn_js_1.BN(1_000_000_000);
/**
 * @name BN_QUINTILL
 * @summary BN constant for 1,000,000,000,000,000,000.
 */
exports.BN_QUINTILL = exports.BN_BILLION.mul(exports.BN_BILLION);
/**
 * @name BN_MAX_INTEGER
 * @summary BN constant for MAX_SAFE_INTEGER
 */
exports.BN_MAX_INTEGER = new bn_js_1.BN(Number.MAX_SAFE_INTEGER);
/**
 * @name BN_SQRT_MAX_INTEGER
 * @summary BN constant for Math.sqrt(MAX_SAFE_INTEGER)
 */
exports.BN_SQRT_MAX_INTEGER = new bn_js_1.BN(94906265);
