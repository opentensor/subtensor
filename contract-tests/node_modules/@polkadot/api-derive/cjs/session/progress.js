"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sessionProgress = exports.eraProgress = exports.eraLength = void 0;
exports.progress = progress;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
function withProgressField(field) {
    return (instanceId, api) => (0, index_js_1.memo)(instanceId, () => api.derive.session.progress().pipe((0, rxjs_1.map)((info) => info[field])));
}
function createDerive(api, info, [currentSlot, epochIndex, epochOrGenesisStartSlot, activeEraStartSessionIndex]) {
    const epochStartSlot = epochIndex.mul(info.sessionLength).iadd(epochOrGenesisStartSlot);
    const sessionProgress = currentSlot.sub(epochStartSlot);
    const eraProgress = info.currentIndex.sub(activeEraStartSessionIndex).imul(info.sessionLength).iadd(sessionProgress);
    return (0, util_1.objectSpread)({
        eraProgress: api.registry.createType('BlockNumber', eraProgress),
        sessionProgress: api.registry.createType('BlockNumber', sessionProgress)
    }, info);
}
function queryAura(api) {
    return api.derive.session.info().pipe((0, rxjs_1.map)((info) => (0, util_1.objectSpread)({
        eraProgress: api.registry.createType('BlockNumber'),
        sessionProgress: api.registry.createType('BlockNumber')
    }, info)));
}
function queryBabe(api) {
    return api.derive.session.info().pipe((0, rxjs_1.switchMap)((info) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(info),
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
    ])), (0, rxjs_1.map)(([info, [currentSlot, epochIndex, genesisSlot, optStartIndex]]) => [
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
function progress(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.babe
        ? queryBabe(api).pipe((0, rxjs_1.map)(([info, slots]) => createDerive(api, info, slots)))
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
exports.eraLength = withProgressField('eraLength');
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
exports.eraProgress = withProgressField('eraProgress');
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
exports.sessionProgress = withProgressField('sessionProgress');
