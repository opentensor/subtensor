"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stakerExposure = void 0;
exports._stakerExposures = _stakerExposures;
exports.stakerExposures = stakerExposures;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function _stakerExposures(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountIds, eras, withActive = false) => {
        const stakerIds = accountIds.map((a) => api.registry.createType('AccountId', a).toString());
        return api.derive.staking._erasExposure(eras, withActive).pipe((0, rxjs_1.map)((exposures) => stakerIds.map((stakerId) => exposures.map(({ era, nominators: allNominators, validators: allValidators }) => {
            const isValidator = !!allValidators[stakerId];
            const validators = {};
            const nominating = allNominators[stakerId] || [];
            if (isValidator) {
                validators[stakerId] = allValidators[stakerId];
            }
            else if (nominating) {
                nominating.forEach(({ validatorId }) => {
                    validators[validatorId] = allValidators[validatorId];
                });
            }
            return { era, isEmpty: !Object.keys(validators).length, isValidator, nominating, validators };
        }))));
    });
}
/**
 * @name stakerExposures
 * @param { (Uint8Array | string)[] } accountIds List of validator stash accounts.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves staking exposure for multiple accounts across historical eras.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.stakerExposures(
 *   [ALICE, BOB],
 *   true
 * );
 * ```
*/
function stakerExposures(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountIds, withActive = false) => api.derive.staking.erasHistoric(withActive).pipe((0, rxjs_1.switchMap)((eras) => api.derive.staking._stakerExposures(accountIds, eras, withActive))));
}
/**
 * @name stakerExposure
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param { boolean } withActive Whether to include the active era.
 * @description Retrieves staking exposure for a single account across historical eras. Exposure refers to the total stake associated with a validator.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.stakerExposure(
 *   ALICE,
 *   true
 * );
 * ```
*/
exports.stakerExposure = (0, index_js_1.firstMemo)((api, accountId, withActive) => api.derive.staking.stakerExposures([accountId], withActive));
