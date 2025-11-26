import { map, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name referendumsFinished
 * @description Retrieves information about finished referendums.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumsFinished();
 * console.log("Finished Referendums:", referendums);
 * ```
 */
export function referendumsFinished(instanceId, api) {
    return memo(instanceId, () => api.derive.democracy.referendumIds().pipe(switchMap((ids) => api.query.democracy.referendumInfoOf.multi(ids)), map((infos) => infos
        .map((o) => o.unwrapOr(null))
        .filter((info) => !!info && info.isFinished)
        .map((info) => info.asFinished))));
}
