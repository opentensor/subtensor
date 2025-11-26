import { map, of } from 'rxjs';
import { assertReturn, isU8a } from '@polkadot/util';
import { decodeAddress } from '@polkadot/util-crypto';
import { memo } from '../util/index.js';
/**
 * @name accountId
 * @param {(Address | AccountId | AccountIndex | string | null)} address An accounts address in various formats.
 * @description Resolves an address (in different formats) to its corresponding `AccountId`.
 * @example
 * ```javascript
 * const ALICE = "F7Hs";
 *
 * api.derive.accounts.accountId(ALICE, (accountId) => {
 *   console.log(`Resolved AccountId: ${accountId}`);
 * });
 * ```
 */
export function accountId(instanceId, api) {
    return memo(instanceId, (address) => {
        const decoded = isU8a(address)
            ? address
            : decodeAddress((address || '').toString());
        if (decoded.length > 8) {
            return of(api.registry.createType(decoded.length === 20 ? 'AccountId20' : 'AccountId', decoded));
        }
        const accountIndex = api.registry.createType('AccountIndex', decoded);
        return api.derive.accounts.indexToId(accountIndex.toString()).pipe(map((a) => assertReturn(a, 'Unable to retrieve accountId')));
    });
}
