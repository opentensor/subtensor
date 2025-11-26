import { combineLatest, map } from 'rxjs';
import { objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
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
export function overview(instanceId, api) {
    return memo(instanceId, () => combineLatest([
        api.derive.session.indexes(),
        api.derive.staking.validators()
    ]).pipe(map(([indexes, { nextElected, validators }]) => objectSpread({}, indexes, {
        nextElected,
        validators
    }))));
}
