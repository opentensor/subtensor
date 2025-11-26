"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.idAndIndex = idAndIndex;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../util/index.js");
/**
 * @name idAndIndex
 * @param {(Address | AccountId | AccountIndex | Uint8Array | string | null)} address An accounts address in various formats.
 * @description An array containing the [[AccountId]] and [[AccountIndex]] as optional values.
 * @example
 * ```javascript
 * api.derive.accounts.idAndIndex('F7Hs', ([id, ix]) => {
 *   console.log(`AccountId #${id} with corresponding AccountIndex ${ix}`);
 * });
 * ```
 */
function idAndIndex(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (address) => {
        try {
            // yes, this can fail, don't care too much, catch will catch it
            const decoded = (0, util_1.isU8a)(address)
                ? address
                : (0, util_crypto_1.decodeAddress)((address || '').toString());
            if (decoded.length > 8) {
                const accountId = api.registry.createType(decoded.length === 20 ? 'AccountId20' : 'AccountId', decoded);
                return api.derive.accounts.idToIndex(accountId).pipe((0, rxjs_1.map)((accountIndex) => [accountId, accountIndex]));
            }
            const accountIndex = api.registry.createType('AccountIndex', decoded);
            return api.derive.accounts.indexToId(accountIndex.toString()).pipe((0, rxjs_1.map)((accountId) => [accountId, accountIndex]));
        }
        catch {
            return (0, rxjs_1.of)([undefined, undefined]);
        }
    });
}
