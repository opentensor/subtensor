import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveEraSlashes } from '../types.js';
export declare function _eraSlashes(instanceId: string, api: DeriveApi): (era: EraIndex, withActive: boolean) => Observable<DeriveEraSlashes>;
/**
 * @name eraSlashes
 * @description Retrieves the slashes for a specific staking era.
 * @param {EraIndex} eras The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const slashes = await api.derive.staking.eraSlashes(era);
 * ```
 */
export declare const eraSlashes: (instanceId: string, api: DeriveApi) => (era: EraIndex) => Observable<DeriveEraSlashes>;
export declare const _erasSlashes: (instanceId: string, api: DeriveApi) => (eras: EraIndex[], withActive: boolean) => Observable<DeriveEraSlashes[]>;
/**
 * @name erasSlashes
 * @description Retrieves slashes for historical eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const slashes = await api.derive.staking.erasSlashes(true);
 * ```
 */
export declare const erasSlashes: (instanceId: string, api: DeriveApi) => (withActive?: boolean) => Observable<DeriveEraSlashes[]>;
