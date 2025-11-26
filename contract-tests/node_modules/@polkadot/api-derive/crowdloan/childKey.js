import { map } from 'rxjs';
import { u8aConcat, u8aToHex } from '@polkadot/util';
import { blake2AsU8a } from '@polkadot/util-crypto';
import { memo } from '../util/index.js';
function createChildKey(info) {
    return u8aToHex(u8aConcat(':child_storage:default:', blake2AsU8a(u8aConcat('crowdloan', (info.fundIndex || info.trieIndex).toU8a()))));
}
/**
 * @name childKey
 * @description Retrieves the child storage key for a given parachain’s crowdloan contributions.
 * This key is used to access contribution data stored in a separate child trie of the blockchain’s state.
 * @param {string | number | BN} paraId The parachain ID for which contributions are being queried.
 * @example
 * ```javascript
 * const childKey = await api.derive.crowdloan.childKey(3369);
 * console.log("Child Key:", childKey);
 * ```
 */
export function childKey(instanceId, api) {
    return memo(instanceId, (paraId) => api.query['crowdloan']['funds'](paraId).pipe(map((optInfo) => optInfo.isSome
        ? createChildKey(optInfo.unwrap())
        : null)));
}
