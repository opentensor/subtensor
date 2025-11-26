import { isObject, isUndefined } from '@polkadot/util';
import { hasEq } from './util.js';
function hasMismatch(a, b) {
    return isUndefined(a) || (hasEq(a)
        ? !a.eq(b)
        : a !== b);
}
function notEntry(value) {
    return !Array.isArray(value) || value.length !== 2;
}
function compareMapArray(a, b) {
    // equal number of entries and each entry in the array should match
    return (a.size === b.length) && !b.some((e) => notEntry(e) ||
        hasMismatch(a.get(e[0]), e[1]));
}
export function compareMap(a, b) {
    if (Array.isArray(b)) {
        return compareMapArray(a, b);
    }
    else if (b instanceof Map) {
        return compareMapArray(a, [...b.entries()]);
    }
    else if (isObject(b)) {
        return compareMapArray(a, Object.entries(b));
    }
    return false;
}
