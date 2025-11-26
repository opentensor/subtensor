import { of, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name referendumsActive
 * @description Retrieves information about active referendums.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumsActive();
 * console.log("Active Referendums:", referendums);
 * ```
 */
export function referendumsActive(instanceId, api) {
    return memo(instanceId, () => api.derive.democracy.referendumIds().pipe(switchMap((ids) => ids.length
        ? api.derive.democracy.referendumsInfo(ids)
        : of([]))));
}
