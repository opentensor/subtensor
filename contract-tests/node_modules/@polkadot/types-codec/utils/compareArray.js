import { isUndefined } from '@polkadot/util';
import { hasEq } from './util.js';
export function compareArray(a, b) {
    if (Array.isArray(b)) {
        return (a.length === b.length) && isUndefined(a.find((v, index) => hasEq(v)
            ? !v.eq(b[index])
            : v !== b[index]));
    }
    return false;
}
