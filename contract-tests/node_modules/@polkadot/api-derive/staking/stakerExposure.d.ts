import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
import type { DeriveStakerExposure } from './types.js';
export declare function _stakerExposures(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], eras: EraIndex[], withActive?: boolean) => Observable<DeriveStakerExposure[][]>;
/**
 * @name stakerExposures
 * @param { (Uint8Array | string)[] } accountIds List of validator stash accounts.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves staking exposure for multiple accounts across historical eras.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.stakerExposures(
 *   [ALICE, BOB],
 *   true
 * );
 * ```
*/
export declare function stakerExposures(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], withActive?: boolean) => Observable<DeriveStakerExposure[][]>;
/**
 * @name stakerExposure
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves staking exposure for a single account across historical eras. Exposure refers to the total stake associated with a validator.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.stakerExposure(
 *   ALICE,
 *   true
 * );
 * ```
*/
export declare const stakerExposure: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, withActive?: boolean | undefined) => Observable<DeriveStakerExposure[]>;
