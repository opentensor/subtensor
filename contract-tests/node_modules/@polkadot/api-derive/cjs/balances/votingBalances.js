"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.votingBalances = votingBalances;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name votingBalances
 * @description Retrieves the balance information for multiple accounts, typically used in governance-related contexts to check voting power.
 * @param {(AccountId | AccountIndex | Address | string)[]} addresses An array of account identifiers.
 * @example
 * ```javascript
 * const addresses = ["5D4b...Zf1", "5HGj...yrV"];
 * const balances = await api.derive.balances.votingBalances(addresses);
 * console.log("Voting Balances:", balances);
 * ```
 */
function votingBalances(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (addresses) => !addresses?.length
        ? (0, rxjs_1.of)([])
        : (0, rxjs_1.combineLatest)(addresses.map((accountId) => api.derive.balances.account(accountId))));
}
