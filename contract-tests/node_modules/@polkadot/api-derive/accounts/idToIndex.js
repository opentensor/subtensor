import { map } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name idToIndex
 * @description Retrieves the corresponding AccountIndex.
 * @param {( AccountId | string )} accountId An accounts Id in different formats.
 * @example
 * ```javascript
 * const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
 * api.derive.accounts.idToIndex(ALICE, (accountIndex) => {
 *   console.log(`The AccountIndex of ${ALICE} is ${accountIndex}`);
 * });
 * ```
 */
export function idToIndex(instanceId, api) {
    return memo(instanceId, (accountId) => api.derive.accounts.indexes().pipe(map((indexes) => indexes[accountId.toString()])));
}
