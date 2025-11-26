import { map } from 'rxjs';
import { memo } from '../util/index.js';
import { erasHistoricApplyAccount } from './util.js';
export function _stakerSlashes(instanceId, api) {
    return memo(instanceId, (accountId, eras, withActive) => {
        const stakerId = api.registry.createType('AccountId', accountId).toString();
        return api.derive.staking._erasSlashes(eras, withActive).pipe(map((slashes) => slashes.map(({ era, nominators, validators }) => ({
            era,
            total: nominators[stakerId] || validators[stakerId] || api.registry.createType('Balance')
        }))));
    });
}
/**
 * @name stakerSlashes
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieve the historical slashes (penalties) for a given staker.
 * @example
 * ```javascript
 *  const stakerSlashes = await api.derive.staking.stakerSlashes(
 *   ALICE, //Alice accountId
 *   true
 * );
 * console.log(
 *   'Staker Slashes:',
 *   stakerSlashes.map(({ era, total }) => `Era ${era}: Slashed ${total.toString()}`)
 * );
 * ```
 */
export const stakerSlashes = /*#__PURE__*/ erasHistoricApplyAccount('_stakerSlashes');
