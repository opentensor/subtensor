import type { Observable } from 'rxjs';
import type { AccountId, AccountIndex } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
/**
 * @name indexToId
 * @description Resolves an AccountIndex (short address) to the full AccountId.
 * @param {( AccountIndex | string )} accountIndex An accounts index in different formats.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const id = await api.derive.accounts.indexToId(ALICE);
 * console.log(id);
 * ```
 */
export declare function indexToId(instanceId: string, api: DeriveApi): (accountIndex: AccountIndex | string) => Observable<AccountId | undefined>;
