import { map, of, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
import { getImageHashBounded } from './util.js';
function withImage(api, nextOpt) {
    if (nextOpt.isNone) {
        return of(null);
    }
    const [hash, threshold] = nextOpt.unwrap();
    return api.derive.democracy.preimage(hash).pipe(map((image) => ({
        image,
        imageHash: getImageHashBounded(hash),
        threshold
    })));
}
/**
 * @name nextExternal
 * @description Retrieves the next external proposal that is scheduled for a referendum.
 * @example
 * ```javascript
 * const nextExternal = await api.derive.democracy.nextExternal();
 * console.log("Next external proposal:", nextExternal);
 * ```
 */
export function nextExternal(instanceId, api) {
    return memo(instanceId, () => api.query.democracy?.nextExternal
        ? api.query.democracy.nextExternal().pipe(switchMap((nextOpt) => withImage(api, nextOpt)))
        : of(null));
}
