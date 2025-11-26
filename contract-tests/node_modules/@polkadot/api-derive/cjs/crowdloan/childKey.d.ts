import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi } from '../types.js';
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
export declare function childKey(instanceId: string, api: DeriveApi): (paraId: string | number | BN) => Observable<string | null>;
