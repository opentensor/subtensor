import { combineLatest, map, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
const DEFAULT_FLAGS = { withController: true, withPrefs: true };
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
export function waitingInfo(instanceId, api) {
    return memo(instanceId, (flags = DEFAULT_FLAGS) => combineLatest([
        api.derive.staking.validators(),
        api.derive.staking.stashes()
    ]).pipe(switchMap(([{ nextElected }, stashes]) => {
        const elected = nextElected.map((a) => a.toString());
        const waiting = stashes.filter((v) => !elected.includes(v.toString()));
        return api.derive.staking.queryMulti(waiting, flags).pipe(map((info) => ({
            info,
            waiting
        })));
    })));
}
