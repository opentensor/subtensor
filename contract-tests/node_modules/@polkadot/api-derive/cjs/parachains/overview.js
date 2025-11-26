"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.overview = overview;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function parse([ids, didUpdate, relayDispatchQueueSizes, infos, pendingSwaps]) {
    return ids.map((id, index) => ({
        didUpdate: (0, util_js_1.didUpdateToBool)(didUpdate, id),
        id,
        info: (0, util_1.objectSpread)({ id }, infos[index].unwrapOr(null)),
        pendingSwapId: pendingSwaps[index].unwrapOr(null),
        relayDispatchQueueSize: relayDispatchQueueSizes[index][0].toNumber()
    }));
}
/**
 * @name overview
 * @description Retrieves an overview of all registered parachains.
 * @example
 * ```javascript
 * await api.derive.parachains.overview((overview) => {
 *   parachains.forEach(parachain => {
 *     console.log(`Parachain ${parachain.id.toString()} is registered.`);
 *   });
 * });
 * ```
 */
function overview(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query['registrar']?.['parachains'] && api.query['parachains']
        ? api.query['registrar']['parachains']().pipe((0, rxjs_1.switchMap)((paraIds) => (0, rxjs_1.combineLatest)([
            (0, rxjs_1.of)(paraIds),
            api.query['parachains']['didUpdate'](),
            api.query['parachains']['relayDispatchQueueSize'].multi(paraIds),
            api.query['registrar']['paras'].multi(paraIds),
            api.query['registrar']['pendingSwap'].multi(paraIds)
        ])), (0, rxjs_1.map)(parse))
        : (0, rxjs_1.of)([]));
}
