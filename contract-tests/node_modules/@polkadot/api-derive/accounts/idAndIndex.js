import { map, of } from 'rxjs';
import { isU8a } from '@polkadot/util';
import { decodeAddress } from '@polkadot/util-crypto';
import { memo } from '../util/index.js';
/**
 * @name idAndIndex
 * @param {(Address | AccountId | AccountIndex | Uint8Array | string | null)} address An accounts address in various formats.
 * @description An array containing the [[AccountId]] and [[AccountIndex]] as optional values.
 * @example
 * ```javascript
 * api.derive.accounts.idAndIndex('F7Hs', ([id, ix]) => {
 *   console.log(`AccountId #${id} with corresponding AccountIndex ${ix}`);
 * });
 * ```
 */
export function idAndIndex(instanceId, api) {
    return memo(instanceId, (address) => {
        try {
            // yes, this can fail, don't care too much, catch will catch it
            const decoded = isU8a(address)
                ? address
                : decodeAddress((address || '').toString());
            if (decoded.length > 8) {
                const accountId = api.registry.createType(decoded.length === 20 ? 'AccountId20' : 'AccountId', decoded);
                return api.derive.accounts.idToIndex(accountId).pipe(map((accountIndex) => [accountId, accountIndex]));
            }
            const accountIndex = api.registry.createType('AccountIndex', decoded);
            return api.derive.accounts.indexToId(accountIndex.toString()).pipe(map((accountId) => [accountId, accountIndex]));
        }
        catch {
            return of([undefined, undefined]);
        }
    });
}
