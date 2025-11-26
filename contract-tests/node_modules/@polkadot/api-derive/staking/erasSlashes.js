import { combineLatest, map, of } from 'rxjs';
import { memo } from '../util/index.js';
import { getEraCache, setEraCache } from './cache.js';
import { combineEras, erasHistoricApply, singleEra } from './util.js';
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
export function _eraSlashes(instanceId, api) {
    return memo(instanceId, (era, withActive) => {
        const [cacheKey, cached] = getEraCache(CACHE_KEY, era, withActive);
        return cached
            ? of(cached)
            : combineLatest([
                api.query.staking.nominatorSlashInEra.entries(era),
                api.query.staking.validatorSlashInEra.entries(era)
            ]).pipe(map(([n, v]) => setEraCache(cacheKey, withActive, mapSlashes(era, n, v))));
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
export const eraSlashes = /*#__PURE__*/ singleEra('_eraSlashes');
export const _erasSlashes = /*#__PURE__*/ combineEras('_eraSlashes');
/**
 * @name erasSlashes
 * @description Retrieves slashes for historical eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const slashes = await api.derive.staking.erasSlashes(true);
 * ```
 */
export const erasSlashes = /*#__PURE__*/ erasHistoricApply('_erasSlashes');
