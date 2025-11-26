import { combineLatest, map, of, switchMap } from 'rxjs';
import { firstMemo, memo } from '../util/index.js';
function extractsIds(stashId, queuedKeys, nextKeys) {
    const sessionIds = (queuedKeys.find(([currentId]) => currentId.eq(stashId)) || [undefined, []])[1];
    const nextSessionIds = nextKeys.unwrapOr([]);
    return {
        nextSessionIds: Array.isArray(nextSessionIds)
            ? nextSessionIds
            : [...nextSessionIds.values()],
        sessionIds: Array.isArray(sessionIds)
            ? sessionIds
            : [...sessionIds.values()]
    };
}
/**
 * @name keys
 * @param { Uint8Array | string } stashId The stash account ID whose session keys are to be retrieved.
 * @description Retrieves the session keys associated with a given stash account.
 * @example
 * ```javascript
 * const keys = await api.derive.staking.keys(
 *   ALICE
 * );
 * console.log(
 *   "Session keys:",
 *   keys.sessionIds.map((key) => `Key: ${key}`)
 * );
 * ```
 */
export const keys = /*#__PURE__*/ firstMemo((api, stashId) => api.derive.staking.keysMulti([stashId]));
/**
 * @name keysMulti
 * @description Retrieves session keys for multiple stash accounts.
 * @param { (Uint8Array | string)[] } stashIds Array of stash account IDs.
 * @example
 * ```javascript
 * const keysMulti = await api.derive.staking.keysMulti([ ALICE, BOB ]);
 * keysMulti.forEach((keys) => {
 *   console.log(
 *     "Session keys:",
 *     keys.sessionIds.map((key) => `Key: ${key}`)
 *   );
 * });
 * ```
 */
export function keysMulti(instanceId, api) {
    return memo(instanceId, (stashIds) => stashIds.length
        ? api.query.session.queuedKeys().pipe(switchMap((queuedKeys) => combineLatest([
            of(queuedKeys),
            api.consts['session']?.['dedupKeyPrefix']
                ? api.query.session.nextKeys.multi(stashIds.map((s) => [api.consts['session']['dedupKeyPrefix'], s]))
                : combineLatest(stashIds.map((s) => api.query.session.nextKeys(s)))
        ])), map(([queuedKeys, nextKeys]) => stashIds.map((stashId, index) => extractsIds(stashId, queuedKeys, nextKeys[index]))))
        : of([]));
}
