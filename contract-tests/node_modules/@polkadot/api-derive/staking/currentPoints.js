import { switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name currentPoints
 * @description Retrieve the staking overview, including elected and points earned.
 * @example
 * ```javascript
 * const currentPoints = await api.derive.staking.currentPoints();
 * console.log(currentPoints.toHuman());
 * ```
 */
export function currentPoints(instanceId, api) {
    return memo(instanceId, () => api.derive.session.indexes().pipe(switchMap(({ activeEra }) => api.query.staking.erasRewardPoints(activeEra))));
}
