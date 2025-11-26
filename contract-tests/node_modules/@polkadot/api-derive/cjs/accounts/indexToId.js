"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.indexToId = indexToId;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name indexToId
 * @description Resolves an AccountIndex (short address) to the full AccountId.
 * @param {( AccountIndex | string )} accountIndex An accounts index in different formats.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const id = await api.derive.accounts.indexToId(ALICE);
 * console.log(id);
 * ```
 */
function indexToId(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountIndex) => api.query.indices
        ? api.query.indices.accounts(accountIndex).pipe((0, rxjs_1.map)((optResult) => optResult.unwrapOr([])[0]))
        : (0, rxjs_1.of)(undefined));
}
