"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bnMin = exports.bnMax = void 0;
const helpers_js_1 = require("../bi/helpers.js");
/**
 * @name bnMax
 * @summary Finds and returns the highest value in an array of BNs.
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnMax } from '@polkadot/util';
 *
 * bnMax([new BN(1), new BN(3), new BN(2)]).toString(); // => '3'
 * ```
 */
exports.bnMax = (0, helpers_js_1.createCmp)((a, b) => a.gt(b));
/**
 * @name bnMin
 * @summary Finds and returns the smallest value in an array of BNs.
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnMin } from '@polkadot/util';
 *
 * bnMin([new BN(1), new BN(3), new BN(2)]).toString(); // => '1'
 * ```
 */
exports.bnMin = (0, helpers_js_1.createCmp)((a, b) => a.lt(b));
