import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveStakingOverview } from '../types.js';
/**
 * @name overview
 * @description Retrieve the staking overview, including elected validators and points earned.
 * @example
 * ```javascript
 * const {
 *   activeEra,
 *   activeEraStart,
 *   currentEra,
 *   currentIndex,
 *   nextElected,
 *   validatorCount,
 *   validators,
 * } = await api.derive.staking.overview();
 * ```
 */
export declare function overview(instanceId: string, api: DeriveApi): () => Observable<DeriveStakingOverview>;
