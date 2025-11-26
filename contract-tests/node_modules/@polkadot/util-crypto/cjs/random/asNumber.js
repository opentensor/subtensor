"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.randomAsNumber = randomAsNumber;
const util_1 = require("@polkadot/util");
const asU8a_js_1 = require("./asU8a.js");
const BN_53 = new util_1.BN(0b11111111111111111111111111111111111111111111111111111);
/**
 * @name randomAsNumber
 * @summary Creates a random number from random bytes.
 * @description
 * Returns a random number generated from the secure bytes.
 * @example
 * <BR>
 *
 * ```javascript
 * import { randomAsNumber } from '@polkadot/util-crypto';
 *
 * randomAsNumber(); // => <random number>
 * ```
 */
function randomAsNumber() {
    return (0, util_1.hexToBn)((0, asU8a_js_1.randomAsHex)(8)).and(BN_53).toNumber();
}
