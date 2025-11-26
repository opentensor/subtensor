import { map, of } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name indexToId
 * @description Resolves an AccountIndex (short address) to the full AccountId.
 * @param {( AccountIndex | string )} accountIndex An accounts index in different formats.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const id = await api.derive.accounts.indexToId(ALICE);
 * console.log(id);
 * ```
 */
export function indexToId(instanceId, api) {
    return memo(instanceId, (accountIndex) => api.query.indices
        ? api.query.indices.accounts(accountIndex).pipe(map((optResult) => optResult.unwrapOr([])[0]))
        : of(undefined));
}
