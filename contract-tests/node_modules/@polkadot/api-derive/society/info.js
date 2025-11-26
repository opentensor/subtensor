import { combineLatest, map, of } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name info
 * @description Get the overall info for a society.
 * @example
 * ```javascript
 * const societyInfo = await api.derive.society.candidates();
 * console.log(societyInfo);
 * ```
 */
export function info(instanceId, api) {
    return memo(instanceId, () => combineLatest([
        api.query.society.bids(),
        api.query.society['defender']
            ? api.query.society['defender']()
            : of(undefined),
        api.query.society.founder(),
        api.query.society.head(),
        api.query.society['maxMembers']
            ? api.query.society['maxMembers']()
            : of(undefined),
        api.query.society.pot()
    ]).pipe(map(([bids, defender, founder, head, maxMembers, pot]) => ({
        bids,
        defender: defender?.unwrapOr(undefined),
        founder: founder.unwrapOr(undefined),
        hasDefender: (defender?.isSome && head.isSome && !head.eq(defender)) || false,
        head: head.unwrapOr(undefined),
        maxMembers,
        pot
    }))));
}
