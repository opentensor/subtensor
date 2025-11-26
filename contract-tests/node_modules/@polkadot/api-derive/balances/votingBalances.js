import { combineLatest, of } from 'rxjs';
import { memo } from '../util/index.js';
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
export function votingBalances(instanceId, api) {
    return memo(instanceId, (addresses) => !addresses?.length
        ? of([])
        : combineLatest(addresses.map((accountId) => api.derive.balances.account(accountId))));
}
