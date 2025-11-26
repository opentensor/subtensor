import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveEraExposurePaged } from '../types.js';
/**
 * erasStakersClipped will be deprecated and replaced with erasStakersPaged. Therefore support is given for both
 * storage queries until erasStakersClipped has been completely out of use.
 */
export declare function _eraExposure(instanceId: string, api: DeriveApi): (era: EraIndex, withActive?: boolean) => Observable<DeriveEraExposurePaged>;
/**
 * @name eraExposure
 * @description Retrieves the staking exposure (nominators and total stake) for a specific era.
 * @param {EraIndex} eras The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const exposure = await api.derive.staking.eraExposure(era);
 * ```
 */
export declare const eraExposure: (instanceId: string, api: DeriveApi) => (era: EraIndex) => Observable<DeriveEraExposurePaged>;
export declare const _erasExposure: (instanceId: string, api: DeriveApi) => (eras: EraIndex[], withActive: boolean) => Observable<DeriveEraExposurePaged[]>;
/**
 * @name erasExposure
 * @description Retrieves staking exposure details for multiple past eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.erasExposure(true);
 * ```
 */
export declare const erasExposure: (instanceId: string, api: DeriveApi) => (withActive?: boolean) => Observable<DeriveEraExposurePaged[]>;
