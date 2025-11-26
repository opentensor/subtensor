"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.idToIndex = idToIndex;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name idToIndex
 * @description Retrieves the corresponding AccountIndex.
 * @param {( AccountId | string )} accountId An accounts Id in different formats.
 * @example
 * ```javascript
 * const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
 * api.derive.accounts.idToIndex(ALICE, (accountIndex) => {
 *   console.log(`The AccountIndex of ${ALICE} is ${accountIndex}`);
 * });
 * ```
 */
function idToIndex(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId) => api.derive.accounts.indexes().pipe((0, rxjs_1.map)((indexes) => indexes[accountId.toString()])));
}
