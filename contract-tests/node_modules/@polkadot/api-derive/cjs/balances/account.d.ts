import type { Observable } from 'rxjs';
import type { AccountId, AccountIndex, Address } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveBalancesAccount } from '../types.js';
/**
 * @name account
 * @description Retrieves the essential balance details for an account, such as free balance and account nonce.
 * @param {( AccountIndex | AccountId | Address | string )} address An accountsId in different formats.
 * @example
 * ```javascript
 * const ALICE = 'F7Hs';
 *
 * api.derive.balances.all(ALICE, ({ accountId, lockedBalance }) => {
 *   console.log(`The account ${accountId} has a locked balance ${lockedBalance} units.`);
 * });
 * ```
 */
export declare function account(instanceId: string, api: DeriveApi): (address: AccountIndex | AccountId | Address | string) => Observable<DeriveBalancesAccount>;
