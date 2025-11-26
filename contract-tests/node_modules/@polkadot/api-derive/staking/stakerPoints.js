import { map } from 'rxjs';
import { memo } from '../util/index.js';
import { erasHistoricApplyAccount } from './util.js';
export function _stakerPoints(instanceId, api) {
    return memo(instanceId, (accountId, eras, withActive) => {
        const stakerId = api.registry.createType('AccountId', accountId).toString();
        return api.derive.staking._erasPoints(eras, withActive).pipe(map((points) => points.map(({ era, eraPoints, validators }) => ({
            era,
            eraPoints,
            points: validators[stakerId] || api.registry.createType('RewardPoint')
        }))));
    });
}
/**
 * @name stakerPoints
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves the era reward points earned by a given staker across all eras.
 * @example
 * ```javascript
 * const points = await api.derive.staking.stakerPoints(
 *   ALICE, //Alice accountId
 *   false
 * );
 * console.log(
 *   'Validator Era Points:',
 *   points.map(({ era, points }) => `Era ${era}: ${points.toString()} points`)
 * );
 * ```
*/
export const stakerPoints = /*#__PURE__*/ erasHistoricApplyAccount('_stakerPoints');
