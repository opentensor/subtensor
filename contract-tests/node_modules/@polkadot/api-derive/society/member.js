import { map } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name member
 * @description Get the member info for a society.
 * @param { AccountId } accountId
 * @example
 * ```javascript
 * const member = await api.derive.society.member(ALICE);
 * console.log(member);
 * ```
 */
export function member(instanceId, api) {
    return memo(instanceId, (accountId) => api.derive.society._members([accountId]).pipe(map(([result]) => result)));
}
