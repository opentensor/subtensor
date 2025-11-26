import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveStakingElected, StakingQueryFlags } from '../types.js';
/**
 * @name electedInfo
 * @param {StakingQueryFlags} flags? (Optional) Query flags to filter the staking data.
 * @param {number} page? (Optional) The page index for paginated results.
 * @description Retrieves detailed staking information about the next elected validators and their associated staking data.
 * @example
 * ```javascript
 * const { nextElected, validators, info } =
 *   await api.derive.staking.electedInfo();
 * console.log(
 *   "Next Elected Validators:",
 *   nextElected.map((acc) => acc.toString())
 * );
 * console.log(
 *   "Current Validators:",
 *   validators.map((acc) => acc.toString())
 * );
 * console.log("Validator Staking Info:", info);
 * ```
 */
export declare function electedInfo(instanceId: string, api: DeriveApi): (flags?: StakingQueryFlags, page?: number) => Observable<DeriveStakingElected>;
