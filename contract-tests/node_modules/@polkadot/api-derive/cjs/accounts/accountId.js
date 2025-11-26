"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.accountId = accountId;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../util/index.js");
/**
 * @name accountId
 * @param {(Address | AccountId | AccountIndex | string | null)} address An accounts address in various formats.
 * @description Resolves an address (in different formats) to its corresponding `AccountId`.
 * @example
 * ```javascript
 * const ALICE = "F7Hs";
 *
 * api.derive.accounts.accountId(ALICE, (accountId) => {
 *   console.log(`Resolved AccountId: ${accountId}`);
 * });
 * ```
 */
function accountId(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (address) => {
        const decoded = (0, util_1.isU8a)(address)
            ? address
            : (0, util_crypto_1.decodeAddress)((address || '').toString());
        if (decoded.length > 8) {
            return (0, rxjs_1.of)(api.registry.createType(decoded.length === 20 ? 'AccountId20' : 'AccountId', decoded));
        }
        const accountIndex = api.registry.createType('AccountIndex', decoded);
        return api.derive.accounts.indexToId(accountIndex.toString()).pipe((0, rxjs_1.map)((a) => (0, util_1.assertReturn)(a, 'Unable to retrieve accountId')));
    });
}
