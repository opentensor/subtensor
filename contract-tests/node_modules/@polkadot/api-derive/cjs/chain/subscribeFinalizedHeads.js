"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports._getHeaderRange = _getHeaderRange;
exports.subscribeFinalizedHeads = subscribeFinalizedHeads;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * Returns a header range from startHash to to (not including) endHash, i.e. lastBlock.parentHash === endHash
 */
function _getHeaderRange(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (startHash, endHash, prev = []) => api.rpc.chain.getHeader(startHash).pipe((0, rxjs_1.switchMap)((header) => header.parentHash.eq(endHash)
        ? (0, rxjs_1.of)([header, ...prev])
        : api.derive.chain._getHeaderRange(header.parentHash, endHash, [header, ...prev]))));
}
/**
 * @name subscribeFinalizedHeads
 * @description An observable of the finalized block headers. Unlike the base
 * chain.subscribeFinalizedHeads this does not skip any headers. Since finalization
 * may skip specific blocks (finalization happens in terms of chains), this version
 * of the derive tracks missing headers (since last retrieved) and provides them
 * to the caller.
 * @example
 * ```javascript
 * const unsub = await api.derive.chain.subscribeFinalizedHeads((finalizedHead) => {
 *   console.log(`${finalizedHead.hash}`);
 * });
 * ```
 */
function subscribeFinalizedHeads(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => {
        let prevHash = null;
        return api.rpc.chain.subscribeFinalizedHeads().pipe((0, rxjs_1.switchMap)((header) => {
            const endHash = prevHash;
            const startHash = header.parentHash;
            prevHash = header.createdAtHash = header.hash;
            return endHash === null || startHash.eq(endHash)
                ? (0, rxjs_1.of)(header)
                : api.derive.chain._getHeaderRange(startHash, endHash, [header]).pipe((0, rxjs_1.switchMap)((headers) => (0, rxjs_1.from)(headers)));
        }));
    });
}
