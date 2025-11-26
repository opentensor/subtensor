import type { Observable } from 'rxjs';
import type { AccountId } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveSocietyMember } from '../types.js';
/**
 * @name member
 * @description Get the member info for a society.
 * @param { AccountId } accountId
 * @example
 * ```javascript
 * const member = await api.derive.society.member(ALICE);
 * console.log(member);
 * ```
 */
export declare function member(instanceId: string, api: DeriveApi): (accountId: AccountId) => Observable<DeriveSocietyMember>;
