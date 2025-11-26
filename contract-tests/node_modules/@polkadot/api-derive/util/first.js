import { map } from 'rxjs';
import { memo } from '@polkadot/rpc-core';
export function firstObservable(obs) {
    return obs.pipe(map(([a]) => a));
}
export function firstMemo(fn) {
    return (instanceId, api) => memo(instanceId, (...args) => firstObservable(fn(api, ...args)));
}
