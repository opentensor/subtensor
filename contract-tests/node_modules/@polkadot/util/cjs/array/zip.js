"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.arrayZip = arrayZip;
/**
 * @name arrayZip
 * @description Combines 2 distinct key/value arrays into a single [K, V] array
 */
function arrayZip(keys, values) {
    const count = keys.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        result[i] = [keys[i], values[i]];
    }
    return result;
}
