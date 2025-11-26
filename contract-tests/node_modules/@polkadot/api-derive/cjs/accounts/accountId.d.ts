import type { Observable } from 'rxjs';
import type { AccountId, AccountIndex, Address } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
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
export declare function accountId(instanceId: string, api: DeriveApi): (address?: Address | AccountId | AccountIndex | string | null) => Observable<AccountId>;
