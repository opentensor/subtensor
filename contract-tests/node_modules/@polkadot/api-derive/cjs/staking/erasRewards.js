"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erasRewards = void 0;
exports._erasRewards = _erasRewards;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const cache_js_1 = require("./cache.js");
const util_js_1 = require("./util.js");
const CACHE_KEY = 'eraRewards';
function mapRewards(eras, optRewards) {
    return eras.map((era, index) => ({
        era,
        eraReward: optRewards[index].unwrapOrDefault()
    }));
}
function _erasRewards(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (eras, withActive) => {
        if (!eras.length) {
            return (0, rxjs_1.of)([]);
        }
        const cached = (0, cache_js_1.getEraMultiCache)(CACHE_KEY, eras, withActive);
        const remaining = (0, util_js_1.filterEras)(eras, cached);
        if (!remaining.length) {
            return (0, rxjs_1.of)(cached);
        }
        return api.query.staking.erasValidatorReward.multi(remaining).pipe((0, rxjs_1.map)((r) => (0, cache_js_1.filterCachedEras)(eras, cached, (0, cache_js_1.setEraMultiCache)(CACHE_KEY, withActive, mapRewards(remaining, r)))));
    });
}
/**
 * @name erasRewards
 * @description Retrieves rewards for historical eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.erasRewards(true);
 * ```
 */
exports.erasRewards = (0, util_js_1.erasHistoricApply)('_erasRewards');
