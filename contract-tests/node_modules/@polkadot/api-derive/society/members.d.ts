import type { Observable } from 'rxjs';
import type { AccountId } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveSocietyMember } from '../types.js';
export declare function _members(instanceId: string, api: DeriveApi): (accountIds: AccountId[]) => Observable<DeriveSocietyMember[]>;
/**
 * @name members
 * @description Get the society members.
 * @example
 * ```javascript
 * const members = await api.derive.society.members();
 * console.log(members);
 * ```
 */
export declare function members(instanceId: string, api: DeriveApi): () => Observable<DeriveSocietyMember[]>;
