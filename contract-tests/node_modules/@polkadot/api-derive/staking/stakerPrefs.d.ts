import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveStakerPrefs } from '../types.js';
export declare function _stakerPrefs(instanceId: string, api: DeriveApi): (accountId: Uint8Array | string, eras: EraIndex[], withActive: boolean) => Observable<DeriveStakerPrefs[]>;
/**
 * @name stakerPrefs
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves the validator preferences for a given staker across historical eras.
 * @example
 * ```javascript
 * const prefs = await api.derive.staking.stakerPrefs(
 *   ALICE, //Alice accountId
 *   false
 * );
 * console.log(
 *   'Validator Preferences:',
 *   prefs.map(
 *     ({ era, validatorPrefs }) => `Era ${era}: Commission ${validatorPrefs.commission.toString()}`
 *   )
 * );
 * ```
*/
export declare const stakerPrefs: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean) => Observable<DeriveStakerPrefs[]>;
