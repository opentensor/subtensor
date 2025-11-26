"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.arrayUnzip = arrayUnzip;
/**
 * @name arrayUnzip
 * @description Splits a single [K, V][] into [K[], V[]]
 */
function arrayUnzip(entries) {
    const count = entries.length;
    const keys = new Array(count);
    const values = new Array(count);
    for (let i = 0; i < count; i++) {
        [keys[i], values[i]] = entries[i];
    }
    return [keys, values];
}
