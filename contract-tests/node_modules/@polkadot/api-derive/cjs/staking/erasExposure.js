"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erasExposure = exports._erasExposure = exports.eraExposure = void 0;
exports._eraExposure = _eraExposure;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const cache_js_1 = require("./cache.js");
const util_js_1 = require("./util.js");
const CACHE_KEY = 'eraExposure';
function mapStakersClipped(era, stakers) {
    const nominators = {};
    const validators = {};
    stakers.forEach(([key, exposure]) => {
        const validatorId = key.args[1].toString();
        validators[validatorId] = exposure;
        exposure.others.forEach(({ who }, validatorIndex) => {
            const nominatorId = who.toString();
            nominators[nominatorId] = nominators[nominatorId] || [];
            nominators[nominatorId].push({ validatorId, validatorIndex });
        });
    });
    return { era, nominators, validators };
}
function mapStakersPaged(era, stakers) {
    const nominators = {};
    const validators = {};
    stakers.forEach(([key, exposureOpt]) => {
        if (exposureOpt.isSome) {
            const validatorId = key.args[1].toString();
            const exposure = exposureOpt.unwrap();
            validators[validatorId] = exposure;
            exposure.others.forEach(({ who }, validatorIndex) => {
                const nominatorId = who.toString();
                nominators[nominatorId] = nominators[nominatorId] || [];
                nominators[nominatorId].push({ validatorId, validatorIndex });
            });
        }
    });
    return { era, nominators, validators };
}
/**
 * erasStakersClipped will be deprecated and replaced with erasStakersPaged. Therefore support is given for both
 * storage queries until erasStakersClipped has been completely out of use.
 */
function _eraExposure(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (era, withActive = false) => {
        const [cacheKey, cached] = (0, cache_js_1.getEraCache)(CACHE_KEY, era, withActive);
        return cached
            ? (0, rxjs_1.of)(cached)
            : api.query.staking.erasStakersPaged
                ? api.query.staking.erasStakersPaged.entries(era).pipe((0, rxjs_1.map)((r) => (0, cache_js_1.setEraCache)(cacheKey, withActive, mapStakersPaged(era, r))))
                : api.query.staking.erasStakersClipped.entries(era).pipe((0, rxjs_1.map)((r) => (0, cache_js_1.setEraCache)(cacheKey, withActive, mapStakersClipped(era, r))));
    });
}
/**
 * @name eraExposure
 * @description Retrieves the staking exposure (nominators and total stake) for a specific era.
 * @param {EraIndex} eras The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const exposure = await api.derive.staking.eraExposure(era);
 * ```
 */
exports.eraExposure = (0, util_js_1.singleEra)('_eraExposure');
exports._erasExposure = (0, util_js_1.combineEras)('_eraExposure');
/**
 * @name erasExposure
 * @description Retrieves staking exposure details for multiple past eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.erasExposure(true);
 * ```
 */
exports.erasExposure = (0, util_js_1.erasHistoricApply)('_erasExposure');
