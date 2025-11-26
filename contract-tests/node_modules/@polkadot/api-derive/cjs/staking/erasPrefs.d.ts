import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveEraPrefs } from '../types.js';
export declare function _eraPrefs(instanceId: string, api: DeriveApi): (era: EraIndex, withActive: boolean) => Observable<DeriveEraPrefs>;
/**
 * @name eraPrefs
 * @description Retrieves the validators commission preferences for a given staking era.
 * @param {EraIndex} era The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const prefs = await api.derive.staking.eraPrefs(era);
 * console.log(JSON.stringify(prefs));
 * ```
 */
export declare const eraPrefs: (instanceId: string, api: DeriveApi) => (era: EraIndex) => Observable<DeriveEraPrefs>;
export declare const _erasPrefs: (instanceId: string, api: DeriveApi) => (eras: EraIndex[], withActive: boolean) => Observable<DeriveEraPrefs[]>;
/**
 * @name erasPrefs
 * @description Retrieves validators commission preferences for multiple past staking eras
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const prefs = await api.derive.staking.erasPrefs(true);
 * ```
 */
export declare const erasPrefs: (instanceId: string, api: DeriveApi) => (withActive?: boolean) => Observable<DeriveEraPrefs[]>;
