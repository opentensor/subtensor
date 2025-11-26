import { map, of } from 'rxjs';
import { memo } from '../util/index.js';
import { getEraCache, setEraCache } from './cache.js';
import { combineEras, erasHistoricApply, singleEra } from './util.js';
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
export function _eraExposure(instanceId, api) {
    return memo(instanceId, (era, withActive = false) => {
        const [cacheKey, cached] = getEraCache(CACHE_KEY, era, withActive);
        return cached
            ? of(cached)
            : api.query.staking.erasStakersPaged
                ? api.query.staking.erasStakersPaged.entries(era).pipe(map((r) => setEraCache(cacheKey, withActive, mapStakersPaged(era, r))))
                : api.query.staking.erasStakersClipped.entries(era).pipe(map((r) => setEraCache(cacheKey, withActive, mapStakersClipped(era, r))));
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
export const eraExposure = /*#__PURE__*/ singleEra('_eraExposure');
export const _erasExposure = /*#__PURE__*/ combineEras('_eraExposure');
/**
 * @name erasExposure
 * @description Retrieves staking exposure details for multiple past eras.
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 * @example
 * ```javascript
 * const exposure = await api.derive.staking.erasExposure(true);
 * ```
 */
export const erasExposure = /*#__PURE__*/ erasHistoricApply('_erasExposure');
