import { combineLatest, map, of, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
function getPrev(api) {
    return api.query.society.candidates().pipe(switchMap((candidates) => combineLatest([
        of(candidates),
        api.query.society['suspendedCandidates'].multi(candidates.map(({ who }) => who))
    ])), map(([candidates, suspended]) => candidates.map(({ kind, value, who }, index) => ({
        accountId: who,
        isSuspended: suspended[index].isSome,
        kind,
        value
    }))));
}
function getCurr(api) {
    return api.query.society.candidates.entries().pipe(map((entries) => entries
        .filter(([, opt]) => opt.isSome)
        .map(([{ args: [accountId] }, opt]) => [accountId, opt.unwrap()])
        // FIXME We are missing the new fields from the candidate record
        .map(([accountId, { bid, kind }]) => ({
        accountId,
        isSuspended: false,
        kind,
        value: bid
    }))));
}
/**
 * @name candidate
 * @description Retrieves the list of candidates for the society module.
 * @example
 * ```javascript
 * const societyCandidates = await api.derive.society.candidates();
 * console.log(societyCandidates);
 * ```
 */
export function candidates(instanceId, api) {
    return memo(instanceId, () => api.query.society['suspendedCandidates'] && api.query.society.candidates.creator.meta.type.isPlain
        ? getPrev(api)
        : getCurr(api));
}
