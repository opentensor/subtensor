import type { Observable } from 'rxjs';
import type { AccountId } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
/**
 * @name stashes
 * @description Retrieve the list of all validator stashes.
 * @example
 * ```javascript
 * const stashes = await api.derive.staking.stashes();
 * console.log(
 *   "Validator Stashes:",
 *   stashes.map((s) => s.toString())
 * );
 * ```
 */
export declare function stashes(instanceId: string, api: DeriveApi): () => Observable<AccountId[]>;
