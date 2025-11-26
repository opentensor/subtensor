import type { Observable } from 'rxjs';
import type { Bytes, Data } from '@polkadot/types';
import type { AccountId } from '@polkadot/types/interfaces';
import type { PalletIdentityRegistration } from '@polkadot/types/lookup';
import type { Option } from '@polkadot/types-codec';
import type { ITuple } from '@polkadot/types-codec/types';
import type { DeriveAccountRegistration, DeriveApi, DeriveHasIdentity } from '../types.js';
export declare function _identity(instanceId: string, api: DeriveApi): (accountId?: AccountId | Uint8Array | string) => Observable<[Option<ITuple<[PalletIdentityRegistration, Option<Bytes>]>> | Option<PalletIdentityRegistration> | undefined, Option<ITuple<[AccountId, Data]>> | undefined]>;
/**
 * @name identity
 * @description Retrieves the on chain identity information for a given account.
 * @param {(AccountId | Uint8Array | string)} accoutId The account identifier to query the identity for.
 * @example
 * ```javascript
 * const ALICE = "13xAUH";
 *
 * api.derive.accounts.identity(ALICE, (identity) => {
 *   console.log(
 *     "Account Identity:",
 *     Object.keys(identity).map((key) => `${key}: ${identity[key]}`)
 *   );
 * });
 * ```
 */
export declare function identity(instanceId: string, api: DeriveApi): (accountId?: AccountId | Uint8Array | string) => Observable<DeriveAccountRegistration>;
/**
 * @name hasIdentity
 * @description Checks if a specific account has an identity registered on chain.
 * @param {(AccountId | Uint8Array | string)} accoutId The account identifier to query.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * console.log(await api.derive.accounts.hasIdentity(ALICE));
 * ```
 */
export declare const hasIdentity: (instanceId: string, api: DeriveApi) => (accountId: string | AccountId | Uint8Array) => Observable<DeriveHasIdentity>;
/**
 * @name hasIdentityMulti
 * @description Checks whether multiple accounts have on chain identities registered.
 * @param {(AccountId | Uint8Array | string)[]} accountIds Array of account identifiers to query.
 * @example
 * ```javascript
 * const ALICE = "13AU";
 * const BOB = "16WW";
 * console.log(await api.derive.accounts.hasIdentityMulti([ALICE, BOB]));
 * ```
 */
export declare function hasIdentityMulti(instanceId: string, api: DeriveApi): (accountIds: (AccountId | Uint8Array | string)[]) => Observable<DeriveHasIdentity[]>;
