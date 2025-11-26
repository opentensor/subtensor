import type { Observable } from 'rxjs';
import type { u32 } from '@polkadot/types';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { AnyNumber } from '@polkadot/types-codec/types';
import type { DeriveApi, DeriveOwnExposure } from '../types.js';
export declare function _ownExposures(instanceId: string, api: DeriveApi): (accountId: Uint8Array | string, eras: EraIndex[], withActive: boolean, page: u32 | AnyNumber) => Observable<DeriveOwnExposure[]>;
/**
 * @name ownExposure
 * @description Retrieves the staking exposure of a validator for a specific era, including their own stake.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param {EraIndex} era The staking era to query.
 * @param { u32 | AnyNumber } page? (Optional) The pagination index.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const exposure = await api.derive.staking.ownExposure(
 *   "11VR4pF6c7kfBhfmuwwjWY3FodeYBKWx7ix2rsRCU2q6hqJ",
 *   era
 * );
 * console.log(JSON.stringify(exposure));
 * ```
 */
export declare const ownExposure: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, era: EraIndex, page?: u32 | AnyNumber | undefined) => Observable<DeriveOwnExposure>;
/**
 * @name ownExposures
 * @description Retrieves staking exposures for a validator across multiple historical eras.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const exposures = await api.derive.staking.ownExposures(
 *   ALICE,
 *   true
 * );
 * ```
 */
export declare const ownExposures: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean) => Observable<DeriveOwnExposure[]>;
