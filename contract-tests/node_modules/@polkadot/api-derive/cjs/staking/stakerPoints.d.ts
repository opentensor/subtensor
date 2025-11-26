import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveStakerPoints } from '../types.js';
export declare function _stakerPoints(instanceId: string, api: DeriveApi): (accountId: Uint8Array | string, eras: EraIndex[], withActive: boolean) => Observable<DeriveStakerPoints[]>;
/**
 * @name stakerPoints
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves the era reward points earned by a given staker across all eras.
 * @example
 * ```javascript
 * const points = await api.derive.staking.stakerPoints(
 *   ALICE, //Alice accountId
 *   false
 * );
 * console.log(
 *   'Validator Era Points:',
 *   points.map(({ era, points }) => `Era ${era}: ${points.toString()} points`)
 * );
 * ```
*/
export declare const stakerPoints: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean) => Observable<DeriveStakerPoints[]>;
