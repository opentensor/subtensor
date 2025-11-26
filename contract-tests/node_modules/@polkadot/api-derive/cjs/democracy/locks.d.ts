import type { Observable } from 'rxjs';
import type { AccountId } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveDemocracyLock } from '../types.js';
/**
 * @name locks
 * @description Retrieves the democracy voting locks for a given account.
 * @param { string | AccountId } accountId The accountId for which to retrieve democracy voting locks.
 * @example
 * ```javascript
 * const locks = await api.derive.democracy.locks('5FfFjX...'); // Replace with an actual accountId
 * console.log("Democracy Locks:", locks);
 * ```
 */
export declare function locks(instanceId: string, api: DeriveApi): (accountId: string | AccountId) => Observable<DeriveDemocracyLock[]>;
