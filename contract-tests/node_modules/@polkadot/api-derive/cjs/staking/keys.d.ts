import type { Observable } from 'rxjs';
import type { DeriveApi } from '../types.js';
import type { DeriveStakingKeys } from './types.js';
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
export declare const keys: (instanceId: string, api: DeriveApi) => (stashId: string | Uint8Array) => Observable<DeriveStakingKeys>;
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
export declare function keysMulti(instanceId: string, api: DeriveApi): (stashIds: (Uint8Array | string)[]) => Observable<DeriveStakingKeys[]>;
