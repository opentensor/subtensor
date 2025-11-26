"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stakerPrefs = void 0;
exports._stakerPrefs = _stakerPrefs;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function _stakerPrefs(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId, eras, _withActive) => api.query.staking.erasValidatorPrefs.multi(eras.map((e) => [e, accountId])).pipe((0, rxjs_1.map)((all) => all.map((validatorPrefs, index) => ({
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
exports.stakerPrefs = (0, util_js_1.erasHistoricApplyAccount)('_stakerPrefs');
