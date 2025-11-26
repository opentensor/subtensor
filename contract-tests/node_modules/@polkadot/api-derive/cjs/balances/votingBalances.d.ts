import type { Observable } from 'rxjs';
import type { AccountId, AccountIndex, Address } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveBalancesAccount } from '../types.js';
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
export declare function votingBalances(instanceId: string, api: DeriveApi): (addresses?: (AccountId | AccountIndex | Address | string)[]) => Observable<DeriveBalancesAccount[]>;
