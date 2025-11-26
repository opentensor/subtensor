import type { Observable } from 'rxjs';
import type { AccountId } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveStakingValidators } from '../types.js';
/**
 * @name nextElected
 * @description Retrieves the list of accounts that are set to be the next elected validators in the staking system. It provides a preview of who will be validators in the next staking era.
 * @example
 * ```javascript
 * const nextElected = await api.derive.staking.nextElected();
 * console.log(
 *   "Next Elected Validators:",
 *   nextElected.map((acc) => acc.toString())
 * );
 * ```
 */
export declare function nextElected(instanceId: string, api: DeriveApi): () => Observable<AccountId[]>;
/**
 * @name validators
 * @description Retrieve latest list of validators.
 * @example
 * ```javascript
 * const { validators, nextElected } = await api.derive.staking.validators();
 * console.log(
 *   "Current Validators:",
 *   validators.map((v) => v.toString())
 * );
 * console.log(
 *   "Next Elected Validators:",
 *   nextElected.map((v) => v.toString())
 * );
 * ```
 */
export declare function validators(instanceId: string, api: DeriveApi): () => Observable<DeriveStakingValidators>;
