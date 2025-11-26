"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erasPrefs = exports._erasPrefs = exports.eraPrefs = void 0;
exports._eraPrefs = _eraPrefs;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const cache_js_1 = require("./cache.js");
const util_js_1 = require("./util.js");
const CACHE_KEY = 'eraPrefs';
function mapPrefs(era, all) {
    const validators = {};
    all.forEach(([key, prefs]) => {
        validators[key.args[1].toString()] = prefs;
    });
    return { era, validators };
}
function _eraPrefs(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (era, withActive) => {
        const [cacheKey, cached] = (0, cache_js_1.getEraCache)(CACHE_KEY, era, withActive);
        return cached
            ? (0, rxjs_1.of)(cached)
            : api.query.staking.erasValidatorPrefs.entries(era).pipe((0, rxjs_1.map)((r) => (0, cache_js_1.setEraCache)(cacheKey, withActive, mapPrefs(era, r))));
    });
}
/**
 * @name eraPrefs
 * @description Retrieves the validators commission preferences for a given staking era.
 * @param {EraIndex} era The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const prefs = await api.derive.staking.eraPrefs(era);
 * console.log(JSON.stringify(prefs));
 * ```
 */
exports.eraPrefs = (0, util_js_1.singleEra)('_eraPrefs');
exports._erasPrefs = (0, util_js_1.combineEras)('_eraPrefs');
/**
 * @name erasPrefs
 * @description Retrieves validators commission preferences for multiple past staking eras
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const prefs = await api.derive.staking.erasPrefs(true);
 * ```
 */
exports.erasPrefs = (0, util_js_1.erasHistoricApply)('_erasPrefs');
