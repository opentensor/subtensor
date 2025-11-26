"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bnSqrt = bnSqrt;
const bn_js_1 = require("./bn.js");
const consts_js_1 = require("./consts.js");
const toBn_js_1 = require("./toBn.js");
/**
 * @name bnSqrt
 * @summary Calculates the integer square root of a BN
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnSqrt } from '@polkadot/util';
 *
 * bnSqrt(new BN(16)).toString(); // => '4'
 * ```
 */
function bnSqrt(value) {
    const n = (0, toBn_js_1.bnToBn)(value);
    if (n.isNeg()) {
        throw new Error('square root of negative numbers is not supported');
    }
    // https://stackoverflow.com/questions/53683995/javascript-big-integer-square-root/
    // shortcut <= 2^53 - 1 to use the JS utils
    if (n.lte(consts_js_1.BN_MAX_INTEGER)) {
        // ~~ More performant version of Math.floor
        return new bn_js_1.BN(~~Math.sqrt(n.toNumber()));
    }
    // Use sqrt(MAX_SAFE_INTEGER) as starting point. since we already know the
    // output will be larger than this, we expect this to be a safe start
    let x0 = consts_js_1.BN_SQRT_MAX_INTEGER.clone();
    while (true) {
        const x1 = n.div(x0).iadd(x0).ishrn(1);
        if (x0.eq(x1) || x0.eq(x1.sub(consts_js_1.BN_ONE))) {
            return x0;
        }
        x0 = x1;
    }
}
