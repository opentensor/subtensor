import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveEraRewards } from '../types.js';
export declare function _erasRewards(instanceId: string, api: DeriveApi): (eras: EraIndex[], withActive: boolean) => Observable<DeriveEraRewards[]>;
/**
 * @name erasRewards
 * @description Retrieves rewards for historical eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.erasRewards(true);
 * ```
 */
export declare const erasRewards: (instanceId: string, api: DeriveApi) => (withActive?: boolean) => Observable<DeriveEraRewards[]>;
