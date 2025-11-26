"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.indexes = indexes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function parse([currentIndex, activeEra, activeEraStart, currentEra, validatorCount]) {
    return {
        activeEra,
        activeEraStart,
        currentEra,
        currentIndex,
        validatorCount
    };
}
function queryStaking(api) {
    return api.queryMulti([
        api.query.session.currentIndex,
        api.query.staking.activeEra,
        api.query.staking.currentEra,
        api.query.staking.validatorCount
    ]).pipe((0, rxjs_1.map)(([currentIndex, activeOpt, currentEra, validatorCount]) => {
        const { index, start } = activeOpt.unwrapOrDefault();
        return parse([
            currentIndex,
            index,
            start,
            currentEra.unwrapOrDefault(),
            validatorCount
        ]);
    }));
}
function querySession(api) {
    return api.query.session.currentIndex().pipe((0, rxjs_1.map)((currentIndex) => parse([
        currentIndex,
        api.registry.createType('EraIndex'),
        api.registry.createType('Option<Moment>'),
        api.registry.createType('EraIndex'),
        api.registry.createType('u32')
    ])));
}
function empty(api) {
    return (0, rxjs_1.of)(parse([
        api.registry.createType('SessionIndex', 1),
        api.registry.createType('EraIndex'),
        api.registry.createType('Option<Moment>'),
        api.registry.createType('EraIndex'),
        api.registry.createType('u32')
    ]));
}
/**
 * @name indexes
 * @description Retrieves session-related index data, adapting to whether
 * the chain has staking enabled.
 * @example
 * ```javascript
 * api.derive.session.indexes((indexes) => {
 *   console.log(`Current session index: ${indexes.currentIndex}`);
 *   console.log(`Validator count: ${indexes.validatorCount}`);
 * });
 * ```
 */
function indexes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.session
        ? api.query.staking
            ? queryStaking(api)
            : querySession(api)
        : empty(api));
}
