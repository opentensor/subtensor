import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveStakingWaiting, StakingQueryFlags } from '../types.js';
/**
 * @name waitingInfo
 * @param {StakingQueryFlags} flags? (Optional) Query flags to filter the staking data.
 * @description Staking candidates who are waiting to become validators.
 * @example
 * ```javascript
 * const { waiting, info } = await api.derive.staking.waitingInfo();
 * console.log(
 *   "Waiting Candidates:",
 *   waiting.map((acc) => acc.toString())
 * );
 * ```
 */
export declare function waitingInfo(instanceId: string, api: DeriveApi): (flags?: StakingQueryFlags) => Observable<DeriveStakingWaiting>;
