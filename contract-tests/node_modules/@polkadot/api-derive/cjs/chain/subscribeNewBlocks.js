"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.subscribeNewBlocks = subscribeNewBlocks;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name subscribeNewBlocks
 * @returns The latest block & events for that block
 * @example
 * ```javascript
 * const unsub = await api.derive.chain.subscribeNewBlocks((newBlock) => {
 *   console.log(`Block Hash: ${newBlock.hash}`);
 * });
 * ```
 */
function subscribeNewBlocks(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.chain.subscribeNewHeads().pipe((0, rxjs_1.switchMap)((header) => api.derive.chain.getBlock(header.createdAtHash || header.hash))));
}
