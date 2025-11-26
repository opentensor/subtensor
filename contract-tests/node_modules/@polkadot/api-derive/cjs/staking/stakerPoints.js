"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stakerPoints = void 0;
exports._stakerPoints = _stakerPoints;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function _stakerPoints(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId, eras, withActive) => {
        const stakerId = api.registry.createType('AccountId', accountId).toString();
        return api.derive.staking._erasPoints(eras, withActive).pipe((0, rxjs_1.map)((points) => points.map(({ era, eraPoints, validators }) => ({
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
exports.stakerPoints = (0, util_js_1.erasHistoricApplyAccount)('_stakerPoints');
