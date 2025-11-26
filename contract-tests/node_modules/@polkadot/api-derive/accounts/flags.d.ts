import type { Observable } from 'rxjs';
import type { AccountId, Address, Balance } from '@polkadot/types/interfaces';
import type { PalletElectionsPhragmenSeatHolder } from '@polkadot/types/lookup';
import type { Option } from '@polkadot/types-codec';
import type { DeriveAccountFlags, DeriveApi } from '../types.js';
type FlagsIntermediate = [
    PalletElectionsPhragmenSeatHolder[] | [AccountId, Balance][] | undefined,
    AccountId[],
    AccountId[],
    AccountId[],
    Option<AccountId> | AccountId | undefined
];
export declare function _flags(instanceId: string, api: DeriveApi): () => Observable<FlagsIntermediate>;
/**
 * @name flags
 * @param {(AccountId | Address | string | null)} address The account identifier.
 * @description Retrieves the membership flags for a given account.
 * @example
 * const ALICE = "F7Hs";
 *
 * api.derive.accounts.flags(ALICE, (flags) => {
 *   console.log(
 *     `Account Flags:`,
 *     Object.keys(flags).map((flag) => `${flag}: ${flags[flag]}`)
 *   );
 * });
 */
export declare function flags(instanceId: string, api: DeriveApi): (address?: AccountId | Address | string | null) => Observable<DeriveAccountFlags>;
export {};
