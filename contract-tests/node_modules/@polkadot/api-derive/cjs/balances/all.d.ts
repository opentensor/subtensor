import type { Observable } from 'rxjs';
import type { AccountId } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveBalancesAll } from '../types.js';
/**
 * @name all
 * @description Retrieves the complete balance information for an account, including free balance, locked balance, reserved balance, and more.
 * @param {( AccountId | string )} address An accountsId in different formats.
 * @example
 * ```javascript
 * const ALICE = 'F7Hs';
 *
 * api.derive.balances.account(ALICE, (accountInfo) => {
 *   console.log(
 *     `${accountInfo.accountId} info:`,
 *     Object.keys(accountInfo).map((key) => `${key}: ${accountInfo[key]}`)
 *   );
 * });
 * ```
 */
export declare function all(instanceId: string, api: DeriveApi): (address: AccountId | string) => Observable<DeriveBalancesAll>;
