"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.subscribeFinalizedBlocks = subscribeFinalizedBlocks;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name subscribeFinalizedBlocks
 * @description Retrieves the finalized block & events for that block
 * @example
 * ```javascript
 * const unsub = await api.derive.chain.subscribeFinalizedBlocks((finalizedBlock) => {
 *  console.log(`# Finalized block ${finalizedBlock.block.hash}`);
 * });
 * ```
 */
function subscribeFinalizedBlocks(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.chain.subscribeFinalizedHeads().pipe((0, rxjs_1.switchMap)((header) => api.derive.chain.getBlock(header.createdAtHash || header.hash))));
}
