import { map } from 'rxjs';
import { memo } from '../util/index.js';
import { erasHistoricApplyAccount } from './util.js';
export function _stakerPrefs(instanceId, api) {
    return memo(instanceId, (accountId, eras, _withActive) => api.query.staking.erasValidatorPrefs.multi(eras.map((e) => [e, accountId])).pipe(map((all) => all.map((validatorPrefs, index) => ({
        era: eras[index],
        validatorPrefs
    })))));
}
/**
 * @name stakerPrefs
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves the validator preferences for a given staker across historical eras.
 * @example
 * ```javascript
 * const prefs = await api.derive.staking.stakerPrefs(
 *   ALICE, //Alice accountId
 *   false
 * );
 * console.log(
 *   'Validator Preferences:',
 *   prefs.map(
 *     ({ era, validatorPrefs }) => `Era ${era}: Commission ${validatorPrefs.commission.toString()}`
 *   )
 * );
 * ```
*/
export const stakerPrefs = /*#__PURE__*/ erasHistoricApplyAccount('_stakerPrefs');
