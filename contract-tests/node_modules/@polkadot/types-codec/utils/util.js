import { isFunction } from '@polkadot/util';
export function hasEq(o) {
    return isFunction(o.eq);
}
