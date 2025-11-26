"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.electedInfo = electedInfo;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const DEFAULT_FLAGS = { withController: true, withExposure: true, withPrefs: true };
function combineAccounts(nextElected, validators) {
    return (0, util_1.arrayFlatten)([nextElected, validators.filter((v) => !nextElected.find((n) => n.eq(v)))]);
}
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
function electedInfo(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (flags = DEFAULT_FLAGS, page = 0) => api.derive.staking.validators().pipe((0, rxjs_1.switchMap)(({ nextElected, validators }) => api.derive.staking.queryMulti(combineAccounts(nextElected, validators), flags, page).pipe((0, rxjs_1.map)((info) => ({
        info,
        nextElected,
        validators
    }))))));
}
