import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveEraPoints, DeriveEraPrefs, DeriveEraRewards, DeriveStakerReward } from '../types.js';
type ErasResult = [DeriveEraPoints[], DeriveEraPrefs[], DeriveEraRewards[]];
export declare function _stakerRewardsEras(instanceId: string, api: DeriveApi): (eras: EraIndex[], withActive?: boolean) => Observable<ErasResult>;
export declare function _stakerRewards(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], eras: EraIndex[], withActive?: boolean) => Observable<DeriveStakerReward[][]>;
/**
 * @name stakerRewards
 * @description Staking rewards history for a given staker.
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.stakerRewards(
 *   ALICE, //Alice accountId
 *   false
 * );
 * ```
 */
export declare const stakerRewards: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean | undefined) => Observable<DeriveStakerReward[]>;
/**
 * @name stakerRewardsMultiEras
 * @description Staking rewards for multiple stakers over specific eras.
 * @param { Uint8Array | string } accountIds List of stakers identified by their AccountId.
 * @param { EraIndex[] } eras Eras for which to retrieve the data.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.stakerRewardsMultiEras(
 *   [ALICE, BOB, CHARLIER], //accountIds
 *   [100,101]  //eras
 * );
 * ```
 */
export declare function stakerRewardsMultiEras(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], eras: EraIndex[]) => Observable<DeriveStakerReward[][]>;
/**
 * @name stakerRewardsMulti
 * @description Staking rewards for multiple stakers.
 * @param { Uint8Array | string } accountIds List of stakers identified by their AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.stakerRewardsMulti(
 *   [ALICE, BOB, CHARLIER], //accountIds
 *   true
 * );
 * ```
 */
export declare function stakerRewardsMulti(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], withActive?: boolean) => Observable<DeriveStakerReward[][]>;
export {};
