import type { Observable } from 'rxjs';
import type { u32 } from '@polkadot/types';
import type { AnyNumber } from '@polkadot/types-codec/types';
import type { DeriveApi, DeriveStakingQuery, StakingQueryFlags } from '../types.js';
/**
 * @name query
 * @param { Uint8Array | string } accountId The stash account to query.
 * @param { StakingQueryFlags } flags Flags to customize the query.
 * @param { u32 } page (Optional) pagination parameter.
 * @description Retrieves staking details for a given stash account.
 * @example
 * ```javascript
 * const stakingInfo = await api.derive.staking.query(
 *   ALICE,
 *   {}
 * );
 * ```
 */
export declare const query: (instanceId: string, api: DeriveApi) => (accountId: string | Uint8Array, flags: StakingQueryFlags, page?: u32 | undefined) => Observable<DeriveStakingQuery>;
/**
 * @name queryMulti
 * @param { (Uint8Array | string)[] } accountIds List of stash accounts to query.
 * @param { StakingQueryFlags } flags Flags to customize the query.
 * @param { u32 } page (Optional) pagination parameter.
 * @description Retrieves staking details for multiple stash accounts.
 * @example
 * ```javascript
 * const stakingInfos = await api.derive.staking.queryMulti([stashId1, stashId2], {});
 * ```
 */
export declare function queryMulti(instanceId: string, api: DeriveApi): (accountIds: (Uint8Array | string)[], flags: StakingQueryFlags, page?: u32 | AnyNumber) => Observable<DeriveStakingQuery[]>;
