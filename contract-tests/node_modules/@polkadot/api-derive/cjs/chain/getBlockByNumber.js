"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getBlockByNumber = getBlockByNumber;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name getBlockByNumber
 * @param {( BN | bigint | Uint8Array | number | string )} blockNumber
 * @description Get a specific block (e.g. rpc.chain.getBlock) and extend it with the author by block number
 * @example
 * ```javascript
 * const { author, block } = await api.derive.chain.getBlockByNumber(123);
 *
 * console.log(`block #${block.header.number} was authored by ${author}`);
 * ```
 */
function getBlockByNumber(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (blockNumber) => api.rpc.chain.getBlockHash(blockNumber).pipe((0, rxjs_1.switchMap)((h) => api.derive.chain.getBlock(h))));
}
