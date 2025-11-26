import type { Observable } from 'rxjs';
import type { AccountId, AccountIndex, Address } from '@polkadot/types/interfaces';
import type { DeriveAccountInfo, DeriveApi } from '../types.js';
/**
 * @name info
 * @description Returns aux. info with regards to an account, current that includes the accountId, accountIndex, identity and nickname
 * @param {(AccountIndex | AccountId | Address | Uint8Array | string | null)} address An accounts in different formats.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const info = await api.derive.accounts.info(ALICE);
 * console.log(
 *   "Account Info: ",
 *   Object.keys(info).map((key) => `${key}: ${info[key]}`)
 * );
 * ```
 */
export declare function info(instanceId: string, api: DeriveApi): (address?: AccountIndex | AccountId | Address | Uint8Array | string | null) => Observable<DeriveAccountInfo>;
