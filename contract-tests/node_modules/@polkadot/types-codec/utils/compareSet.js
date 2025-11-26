import { isObject } from '@polkadot/util';
function compareSetArray(a, b) {
    // equal number of entries and each entry in the array should match
    return (a.size === b.length) && !b.some((e) => !a.has(e));
}
export function compareSet(a, b) {
    if (Array.isArray(b)) {
        return compareSetArray(a, b);
    }
    else if (b instanceof Set) {
        return compareSetArray(a, [...b.values()]);
    }
    else if (isObject(b)) {
        return compareSetArray(a, Object.values(b));
    }
    return false;
}
