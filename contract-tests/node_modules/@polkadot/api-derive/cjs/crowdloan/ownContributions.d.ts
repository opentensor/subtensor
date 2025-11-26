import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi, DeriveOwnContributions } from '../types.js';
/**
 * @name ownContributions
 * @description Retrieves the contribution amounts made by specific accounts (`keys`) to a given parachain crowdloan (`paraId`).
 * @param {string | number | BN} paraId The parachain ID for which contributions are being queried.
 * @param {string[]} keys An array of account addresses whose contributions are to be fetched.
 * @example
 * ```javascript
 * const contributions = await api.derive.crowdloan.ownContributions(2000, ['5Ff...PqV', '5Gg...XyZ']);
 * console.log("Own Contributions:", contributions);
 * ```
 */
export declare function ownContributions(instanceId: string, api: DeriveApi): (paraId: string | number | BN, keys: string[]) => Observable<DeriveOwnContributions>;
