"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.childKey = childKey;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../util/index.js");
function createChildKey(info) {
    return (0, util_1.u8aToHex)((0, util_1.u8aConcat)(':child_storage:default:', (0, util_crypto_1.blake2AsU8a)((0, util_1.u8aConcat)('crowdloan', (info.fundIndex || info.trieIndex).toU8a()))));
}
/**
 * @name childKey
 * @description Retrieves the child storage key for a given parachain’s crowdloan contributions.
 * This key is used to access contribution data stored in a separate child trie of the blockchain’s state.
 * @param {string | number | BN} paraId The parachain ID for which contributions are being queried.
 * @example
 * ```javascript
 * const childKey = await api.derive.crowdloan.childKey(3369);
 * console.log("Child Key:", childKey);
 * ```
 */
function childKey(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (paraId) => api.query['crowdloan']['funds'](paraId).pipe((0, rxjs_1.map)((optInfo) => optInfo.isSome
        ? createChildKey(optInfo.unwrap())
        : null)));
}
