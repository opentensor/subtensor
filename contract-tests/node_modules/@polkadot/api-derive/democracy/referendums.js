import { combineLatest, map, of, switchMap } from 'rxjs';
import { objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
/**
 * @name referendums
 * @description Retrieves information about all active referendums, including their details and associated votes.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendums();
 * ```
 */
export function referendums(instanceId, api) {
    return memo(instanceId, () => api.derive.democracy.referendumsActive().pipe(switchMap((referendums) => referendums.length
        ? combineLatest([
            of(referendums),
            api.derive.democracy._referendumsVotes(referendums)
        ])
        : of([[], []])), map(([referendums, votes]) => referendums.map((referendum, index) => objectSpread({}, referendum, votes[index])))));
}
