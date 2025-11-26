import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveStakingAccount } from '../types.js';
import type { StakingQueryFlags } from './types.js';
/**
 * @name accounts
 * @param {(Uint8Array | string)[]} accountIds List of account stashes
 * @param {StakingQueryFlags} opts optional filtering flag
 * @description From a list of stashes, fill in all the relevant staking details
 * @example
 * ```javascript
 * const accounts = await api.derive.staking.accounts([
 *  "149B17nn7zVL4SkLSNmANupEkGexUBAxVrdk4bbWFZYibkFc",
 * ]);
 * console.log("First account staking info:", accounts[0]);
 * ```
 */
export declare function accounts(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], opts?: StakingQueryFlags) => Observable<DeriveStakingAccount[]>;
/**
 * @name account
 * @param {(Uint8Array | string)} accountId AccountId of the stash.
 * @param {StakingQueryFlags} opts (Optional) filtering flag.
 * @description From a stash, retrieve the controllerId and fill in all the relevant staking details.
 * @example
 * ```javascript
 * const accountStakingData = await api.derive.staking.account(
 *   "149B17nn7zVL4SkLSNmANupEkGexUBAxVrdk4bbWFZYibkFc"
 * );
 * console.log(accountStakingData);
 * ```
 */
export declare const account: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, opts?: StakingQueryFlags | undefined) => Observable<DeriveStakingAccount>;
