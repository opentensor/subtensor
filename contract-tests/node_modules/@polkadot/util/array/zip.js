/**
 * @name arrayZip
 * @description Combines 2 distinct key/value arrays into a single [K, V] array
 */
export function arrayZip(keys, values) {
    const count = keys.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        result[i] = [keys[i], values[i]];
    }
    return result;
}
