import { map, of } from 'rxjs';
import { memo } from '../util/index.js';
import { getEraCache, setEraCache } from './cache.js';
import { combineEras, erasHistoricApply, singleEra } from './util.js';
const CACHE_KEY = 'eraPrefs';
function mapPrefs(era, all) {
    const validators = {};
    all.forEach(([key, prefs]) => {
        validators[key.args[1].toString()] = prefs;
    });
    return { era, validators };
}
export function _eraPrefs(instanceId, api) {
    return memo(instanceId, (era, withActive) => {
        const [cacheKey, cached] = getEraCache(CACHE_KEY, era, withActive);
        return cached
            ? of(cached)
            : api.query.staking.erasValidatorPrefs.entries(era).pipe(map((r) => setEraCache(cacheKey, withActive, mapPrefs(era, r))));
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
export const eraPrefs = /*#__PURE__*/ singleEra('_eraPrefs');
export const _erasPrefs = /*#__PURE__*/ combineEras('_eraPrefs');
/**
 * @name erasPrefs
 * @description Retrieves validators commission preferences for multiple past staking eras
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const prefs = await api.derive.staking.erasPrefs(true);
 * ```
 */
export const erasPrefs = /*#__PURE__*/ erasHistoricApply('_erasPrefs');
