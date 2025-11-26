import type { Observable } from 'rxjs';
import type { EraIndex } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveEraPoints } from '../types.js';
export declare function _erasPoints(instanceId: string, api: DeriveApi): (eras: EraIndex[], withActive: boolean) => Observable<DeriveEraPoints[]>;
/**
 * @name erasPoints
 * @description Retrieves historical era points with its validators.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const points = await api.derive.staking.erasPoints(true);
 * console.log(
 *   "Validator points:",
 *   points.map(({ era, eraPoints }) => `Era: ${era}, points ${eraPoints}`)
 * );
 * ```
 */
export declare const erasPoints: (instanceId: string, api: DeriveApi) => (withActive?: boolean) => Observable<DeriveEraPoints[]>;
