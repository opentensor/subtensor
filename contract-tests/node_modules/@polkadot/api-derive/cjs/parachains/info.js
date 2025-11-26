"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.info = info;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function parseActive(id, active) {
    const found = active.find(([paraId]) => paraId === id);
    if (found && found[1].isSome) {
        const [collatorId, retriable] = found[1].unwrap();
        return (0, util_1.objectSpread)({ collatorId }, retriable.isWithRetries
            ? {
                isRetriable: true,
                retries: retriable.asWithRetries.toNumber()
            }
            : {
                isRetriable: false,
                retries: 0
            });
    }
    return null;
}
function parseCollators(id, collatorQueue) {
    return collatorQueue.map((queue) => {
        const found = queue.find(([paraId]) => paraId === id);
        return found ? found[1] : null;
    });
}
function parse(id, [active, retryQueue, selectedThreads, didUpdate, info, pendingSwap, heads, relayDispatchQueue]) {
    if (info.isNone) {
        return null;
    }
    return {
        active: parseActive(id, active),
        didUpdate: (0, util_js_1.didUpdateToBool)(didUpdate, id),
        heads,
        id,
        info: (0, util_1.objectSpread)({ id }, info.unwrap()),
        pendingSwapId: pendingSwap.unwrapOr(null),
        relayDispatchQueue,
        retryCollators: parseCollators(id, retryQueue),
        selectedCollators: parseCollators(id, selectedThreads)
    };
}
/**
 * @name info
 * @param {ParaId | number} id A unique numeric (non-negative integer) identifier for a parachain.
 * @description Retrieves detailed information about a specific parachain.
 * @example
 * ```javascript
 * await api.derive.parachains.info(1000, (info) => {
 *   if (info) {
 *     console.log(`Parachain ${info.id.toString()} is active: ${info.active}`);
 *   } else {
 *     console.log("Parachain information not available.");
 *   }
 * });
 * ```
*/
function info(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (id) => api.query['registrar'] && api.query['parachains']
        ? api.queryMulti([
            api.query['registrar']['active'],
            api.query['registrar']['retryQueue'],
            api.query['registrar']['selectedThreads'],
            api.query['parachains']['didUpdate'],
            [api.query['registrar']['paras'], id],
            [api.query['registrar']['pendingSwap'], id],
            [api.query['parachains']['heads'], id],
            [api.query['parachains']['relayDispatchQueue'], id]
        ])
            .pipe((0, rxjs_1.map)((result) => parse(api.registry.createType('ParaId', id), result)))
        : (0, rxjs_1.of)(null));
}
