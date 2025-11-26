import { isFunction } from '@polkadot/util';
export function isKeyringPair(account) {
    return isFunction(account.sign);
}
