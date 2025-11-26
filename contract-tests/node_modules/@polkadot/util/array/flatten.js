/**
 * @name arrayFlatten
 * @summary Merge T[][] into T[]
 * @description
 * Returns a new array with all arrays merged into one
 * @example
 * <BR>
 *
 * ```javascript
 * import { arrayFlatten } from '@polkadot/util';
 *
 * arrayFlatten([[1, 2], [3, 4], [5]]); // [1, 2, 3, 4, 5]
 * ```
 */
export function arrayFlatten(arrays) {
    const num = arrays.length;
    // shortcuts for the empty & single-entry case
    if (num === 0) {
        return [];
    }
    else if (num === 1) {
        return arrays[0];
    }
    // pre-allocate based on the combined size
    let size = 0;
    for (let i = 0; i < num; i++) {
        size += arrays[i].length;
    }
    const output = new Array(size);
    let i = -1;
    for (let j = 0; j < num; j++) {
        const a = arrays[j];
        // instead of pushing, we just set the entries
        for (let e = 0, count = a.length; e < count; e++) {
            output[++i] = a[e];
        }
    }
    return output;
}
