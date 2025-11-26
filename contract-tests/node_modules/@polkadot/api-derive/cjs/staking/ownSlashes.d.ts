import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveStakerSlashes } from '../types.js';
export declare function _ownSlashes(instanceId: string, api: DeriveApi): (accountId: Uint8Array | string, eras: EraIndex[], withActive: boolean) => Observable<DeriveStakerSlashes[]>;
/**
 * @name ownSlash
 * @description Retrieves the slashes applied to a specific account in a given era.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param {EraIndex} era The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const slashedAmount = await api.derive.staking.ownSlash(
 *   ALICE,
 *   era
 * );
 * console.log(`Era: ${slashedAmount.era}, total ${slashedAmount.total}`);
 * ```
 */
export declare const ownSlash: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, era: EraIndex) => Observable<DeriveStakerSlashes>;
/**
 * @name ownSlashes
 * @description Retrieves the slashes for a specific account across all historic eras.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const slashes = await api.derive.staking.ownSlashes(
 *   ALICE,
 *   true
 * );
 * console.log(slashes);
 * ```
 */
export declare const ownSlashes: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean) => Observable<DeriveStakerSlashes[]>;
