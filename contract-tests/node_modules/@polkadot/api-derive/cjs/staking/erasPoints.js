"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erasPoints = void 0;
exports._erasPoints = _erasPoints;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const cache_js_1 = require("./cache.js");
const util_js_1 = require("./util.js");
const CACHE_KEY = 'eraPoints';
function mapValidators({ individual }) {
    return [...individual.entries()]
        .filter(([, points]) => points.gt(util_1.BN_ZERO))
        .reduce((result, [validatorId, points]) => {
        result[validatorId.toString()] = points;
        return result;
    }, {});
}
function mapPoints(eras, points) {
    return eras.map((era, index) => ({
        era,
        eraPoints: points[index].total,
        validators: mapValidators(points[index])
    }));
}
function _erasPoints(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (eras, withActive) => {
        if (!eras.length) {
            return (0, rxjs_1.of)([]);
        }
        const cached = (0, cache_js_1.getEraMultiCache)(CACHE_KEY, eras, withActive);
        const remaining = (0, util_js_1.filterEras)(eras, cached);
        return !remaining.length
            ? (0, rxjs_1.of)(cached)
            : api.query.staking.erasRewardPoints.multi(remaining).pipe((0, rxjs_1.map)((p) => (0, cache_js_1.filterCachedEras)(eras, cached, (0, cache_js_1.setEraMultiCache)(CACHE_KEY, withActive, mapPoints(remaining, p)))));
    });
}
/**
 * @name erasPoints
 * @description Retrieves historical era points with its validators.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const points = await api.derive.staking.erasPoints(true);
 * console.log(
 *   "Validator points:",
 *   points.map(({ era, eraPoints }) => `Era: ${era}, points ${eraPoints}`)
 * );
 * ```
 */
exports.erasPoints = (0, util_js_1.erasHistoricApply)('_erasPoints');
