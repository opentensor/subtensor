"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stakerSlashes = void 0;
exports._stakerSlashes = _stakerSlashes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function _stakerSlashes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId, eras, withActive) => {
        const stakerId = api.registry.createType('AccountId', accountId).toString();
        return api.derive.staking._erasSlashes(eras, withActive).pipe((0, rxjs_1.map)((slashes) => slashes.map(({ era, nominators, validators }) => ({
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
exports.stakerSlashes = (0, util_js_1.erasHistoricApplyAccount)('_stakerSlashes');
