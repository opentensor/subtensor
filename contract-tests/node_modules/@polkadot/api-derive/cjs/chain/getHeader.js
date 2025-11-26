"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getHeader = getHeader;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../type/index.js");
const index_js_2 = require("../util/index.js");
const util_js_1 = require("./util.js");
/**
 * @name getHeader
 * @param {( Uint8Array | string )} hash - A block hash as U8 array or string.
 * @returns An array containing the block header and the block author
 * @description Get a specific block header and extend it with the author
 * @example
 * ```javascript
 * const { author, number } = await api.derive.chain.getHeader('0x123...456');
 *
 * console.log(`block #${number} was authored by ${author}`);
 * ```
 */
function getHeader(instanceId, api) {
    return (0, index_js_2.memo)(instanceId, (blockHash) => api.rpc.chain.getHeader(blockHash).pipe((0, rxjs_1.switchMap)((header) => (0, util_js_1.getAuthorDetails)(api, header, blockHash)), (0, rxjs_1.map)(([header, validators, author]) => (0, index_js_1.createHeaderExtended)((validators || header).registry, header, validators, author))));
}
