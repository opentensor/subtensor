import { Observable } from 'rxjs';
import { memoize } from '@polkadot/util';
import { drr } from './drr.js';
/** @internal */
export function memo(instanceId, inner) {
    const options = { getInstanceId: () => instanceId };
    const cached = memoize((...params) => new Observable((observer) => {
        const subscription = inner(...params).subscribe(observer);
        return () => {
            cached.unmemoize(...params);
            subscription.unsubscribe();
        };
    }).pipe(drr()), options);
    return cached;
}
