"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.subscribeNewHeads = subscribeNewHeads;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../type/index.js");
const index_js_2 = require("../util/index.js");
const util_js_1 = require("./util.js");
/**
 * @name subscribeNewHeads
 * @returns A header with the current header (including extracted author).
 * @description An observable of the current block header and it's author.
 * @example
 * ```javascript
 * api.derive.chain.subscribeNewHeads((header) => {
 *   console.log(`block #${header.number} was authored by ${header.author}`);
 * });
 * ```
 */
function subscribeNewHeads(instanceId, api) {
    return (0, index_js_2.memo)(instanceId, () => api.rpc.chain.subscribeNewHeads().pipe((0, rxjs_1.switchMap)((header) => (0, util_js_1.getAuthorDetails)(api, header)), (0, rxjs_1.map)(([header, validators, author]) => {
        header.createdAtHash = header.hash;
        return (0, index_js_1.createHeaderExtended)(header.registry, header, validators, author);
    })));
}
