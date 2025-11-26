"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nSqrt = nSqrt;
const x_bigint_1 = require("@polkadot/x-bigint");
const consts_js_1 = require("./consts.js");
const toBigInt_js_1 = require("./toBigInt.js");
/**
 * @name nSqrt
 * @summary Calculates the integer square root of a bigint
 */
function nSqrt(value) {
    const n = (0, toBigInt_js_1.nToBigInt)(value);
    if (n < consts_js_1._0n) {
        throw new Error('square root of negative numbers is not supported');
    }
    // https://stackoverflow.com/questions/53683995/javascript-big-integer-square-root/
    // shortcut <= 2^53 - 1 to use the JS utils
    if (n <= consts_js_1._2pow53n) {
        // ~~ is more performant that Math.floor
        return (0, x_bigint_1.BigInt)(~~Math.sqrt(Number(n)));
    }
    // Use sqrt(MAX_SAFE_INTEGER) as starting point. since we already know the
    // output will be larger than this, we expect this to be a safe start
    let x0 = consts_js_1._sqrt2pow53n;
    while (true) {
        const x1 = ((n / x0) + x0) >> consts_js_1._1n;
        if (x0 === x1 || (x0 === (x1 - consts_js_1._1n))) {
            return x0;
        }
        x0 = x1;
    }
}
