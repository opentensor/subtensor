import { combineLatest, map, of, switchMap } from 'rxjs';
import { objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
function withProgressField(field) {
    return (instanceId, api) => memo(instanceId, () => api.derive.session.progress().pipe(map((info) => info[field])));
}
function createDerive(api, info, [currentSlot, epochIndex, epochOrGenesisStartSlot, activeEraStartSessionIndex]) {
    const epochStartSlot = epochIndex.mul(info.sessionLength).iadd(epochOrGenesisStartSlot);
    const sessionProgress = currentSlot.sub(epochStartSlot);
    const eraProgress = info.currentIndex.sub(activeEraStartSessionIndex).imul(info.sessionLength).iadd(sessionProgress);
    return objectSpread({
        eraProgress: api.registry.createType('BlockNumber', eraProgress),
        sessionProgress: api.registry.createType('BlockNumber', sessionProgress)
    }, info);
}
function queryAura(api) {
    return api.derive.session.info().pipe(map((info) => objectSpread({
        eraProgress: api.registry.createType('BlockNumber'),
        sessionProgress: api.registry.createType('BlockNumber')
    }, info)));
}
function queryBabe(api) {
    return api.derive.session.info().pipe(switchMap((info) => combineLatest([
        of(info),
        // we may have no staking, but have babe (permissioned)
        api.query.staking?.erasStartSessionIndex
            ? api.queryMulti([
                api.query.babe.currentSlot,
                api.query.babe.epochIndex,
                api.query.babe.genesisSlot,
                [api.query.staking.erasStartSessionIndex, info.activeEra]
            ])
            : api.queryMulti([
                api.query.babe.currentSlot,
                api.query.babe.epochIndex,
                api.query.babe.genesisSlot
            ])
    ])), map(([info, [currentSlot, epochIndex, genesisSlot, optStartIndex]]) => [
        info, [currentSlot, epochIndex, genesisSlot, optStartIndex && optStartIndex.isSome ? optStartIndex.unwrap() : api.registry.createType('SessionIndex', 1)]
    ]));
}
/**
 * @name progress
 * @description Retrieves session information and progress.
 * @example
 * ```javascript
 * api.derive.session.progress((progress) => {
 *   console.log(`Session progress ${JSON.stringify(progress)}`);
 * });
 * ```
 */
export function progress(instanceId, api) {
    return memo(instanceId, () => api.query.babe
        ? queryBabe(api).pipe(map(([info, slots]) => createDerive(api, info, slots)))
        : queryAura(api));
}
/**
 * @name eraLenght
 * @description Retrieves the total length of the current era.
 * @example
 * ```javascript
 * api.derive.session.eraLength((length) => {
 *   console.log(`Current era length: ${length} sessions`);
 * });
 * ```
 */
export const eraLength = /*#__PURE__*/ withProgressField('eraLength');
/**
 * @name eraProgress
 * @description Retrieves the progress of the current era.
 * @example
 * ```javascript
 * api.derive.session.eraProgress((progress) => {
 *   console.log(`Current era progress: ${progress} sessions`);
 * });
 * ```
 */
export const eraProgress = /*#__PURE__*/ withProgressField('eraProgress');
/**
 * @name sessionProgress
 * @description Retrieves the progress of the current session.
 * @example
 * ```javascript
 *   api.derive.session.sessionProgress((progress) => {
 *   console.log(`Current session progress: ${progress} slots`);
 * });
 * ```
 */
export const sessionProgress = /*#__PURE__*/ withProgressField('sessionProgress');
