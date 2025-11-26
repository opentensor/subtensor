"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.arrayShuffle = arrayShuffle;
/**
 * @name arrayShuffle
 * @description Shuffles the input array (unlike sort, this is not done in-place)
 */
function arrayShuffle(input) {
    const result = input.slice();
    let curr = result.length;
    // noop for the single entry
    if (curr === 1) {
        return result;
    }
    while (curr !== 0) {
        // ~~ is more performant than Math.floor
        const rand = ~~(Math.random() * curr);
        curr--;
        [result[curr], result[rand]] = [result[rand], result[curr]];
    }
    return result;
}
