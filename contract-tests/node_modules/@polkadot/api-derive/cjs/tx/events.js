"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.events = events;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name events
 * @param {Hash} at Block hash to query at.
 * @description Retrieves the block information alongside its events at a given block hash
 * @example
 * ```javascript
 * const blockHash = api.registry.createType(
 *   "Hash",
 *   "0xf1dc2efe8265be67deea5e91b05a98a7f9f81f66854e92825cf36f541beb7af6"
 * );
 * const { events, block } = await api.derive.tx.events(blockHash);
 * ```
 */
function events(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (blockHash) => (0, rxjs_1.combineLatest)([
        api.rpc.chain.getBlock(blockHash),
        api.queryAt(blockHash).pipe((0, rxjs_1.switchMap)((queryAt) => queryAt.system.events()))
    ]).pipe((0, rxjs_1.map)(([block, events]) => ({ block, events }))));
}
