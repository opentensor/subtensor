"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erasSlashes = exports._erasSlashes = exports.eraSlashes = void 0;
exports._eraSlashes = _eraSlashes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const cache_js_1 = require("./cache.js");
const util_js_1 = require("./util.js");
const CACHE_KEY = 'eraSlashes';
function mapSlashes(era, noms, vals) {
    const nominators = {};
    const validators = {};
    noms.forEach(([key, optBalance]) => {
        nominators[key.args[1].toString()] = optBalance.unwrap();
    });
    vals.forEach(([key, optRes]) => {
        validators[key.args[1].toString()] = optRes.unwrapOrDefault()[1];
    });
    return { era, nominators, validators };
}
function _eraSlashes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (era, withActive) => {
        const [cacheKey, cached] = (0, cache_js_1.getEraCache)(CACHE_KEY, era, withActive);
        return cached
            ? (0, rxjs_1.of)(cached)
            : (0, rxjs_1.combineLatest)([
                api.query.staking.nominatorSlashInEra.entries(era),
                api.query.staking.validatorSlashInEra.entries(era)
            ]).pipe((0, rxjs_1.map)(([n, v]) => (0, cache_js_1.setEraCache)(cacheKey, withActive, mapSlashes(era, n, v))));
    });
}
/**
 * @name eraSlashes
 * @description Retrieves the slashes for a specific staking era.
 * @param {EraIndex} eras The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const slashes = await api.derive.staking.eraSlashes(era);
 * ```
 */
exports.eraSlashes = (0, util_js_1.singleEra)('_eraSlashes');
exports._erasSlashes = (0, util_js_1.combineEras)('_eraSlashes');
/**
 * @name erasSlashes
 * @description Retrieves slashes for historical eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const slashes = await api.derive.staking.erasSlashes(true);
 * ```
 */
exports.erasSlashes = (0, util_js_1.erasHistoricApply)('_erasSlashes');
