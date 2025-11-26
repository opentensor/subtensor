"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getBlock = getBlock;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../type/index.js");
const index_js_2 = require("../util/index.js");
const util_js_1 = require("./util.js");
/**
 * @name getBlock
 * @param {( Uint8Array | string )} hash A block hash as U8 array or string.
 * @description Get a specific block (e.g. rpc.chain.getBlock) and extend it with the author
 * @example
 * ```javascript
 * const { author, block } = await api.derive.chain.getBlock('0x123...456');
 *
 * console.log(`block #${block.header.number} was authored by ${author}`);
 * ```
 */
function getBlock(instanceId, api) {
    return (0, index_js_2.memo)(instanceId, (blockHash) => (0, rxjs_1.combineLatest)([
        api.rpc.chain.getBlock(blockHash),
        api.queryAt(blockHash)
    ]).pipe((0, rxjs_1.switchMap)(([signedBlock, queryAt]) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(signedBlock),
        queryAt.system.events(),
        (0, util_js_1.getAuthorDetails)(api, signedBlock.block.header, blockHash)
    ])), (0, rxjs_1.map)(([signedBlock, events, [, validators, author]]) => (0, index_js_1.createSignedBlockExtended)(events.registry, signedBlock, events, validators, author))));
}
