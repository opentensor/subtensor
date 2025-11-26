import { map, of } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name referendumIds
 * @description Retrieves an array of active referendum IDs.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumIds();
 * ```
 */
export function referendumIds(instanceId, api) {
    return memo(instanceId, () => api.query.democracy?.lowestUnbaked
        ? api.queryMulti([
            api.query.democracy.lowestUnbaked,
            api.query.democracy.referendumCount
        ]).pipe(map(([first, total]) => total.gt(first)
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            ? [...Array(total.sub(first).toNumber())].map((_, i) => first.addn(i))
            : []))
        : of([]));
}
