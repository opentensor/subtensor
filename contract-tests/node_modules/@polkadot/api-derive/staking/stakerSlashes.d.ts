import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveStakerSlashes } from '../types.js';
export declare function _stakerSlashes(instanceId: string, api: DeriveApi): (accountId: Uint8Array | string, eras: EraIndex[], withActive: boolean) => Observable<DeriveStakerSlashes[]>;
/**
 * @name stakerSlashes
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieve the historical slashes (penalties) for a given staker.
 * @example
 * ```javascript
 *  const stakerSlashes = await api.derive.staking.stakerSlashes(
 *   ALICE, //Alice accountId
 *   true
 * );
 * console.log(
 *   'Staker Slashes:',
 *   stakerSlashes.map(({ era, total }) => `Era ${era}: Slashed ${total.toString()}`)
 * );
 * ```
 */
export declare const stakerSlashes: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean) => Observable<DeriveStakerSlashes[]>;
