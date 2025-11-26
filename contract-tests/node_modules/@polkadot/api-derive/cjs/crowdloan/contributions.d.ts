import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi, DeriveContributions } from '../types.js';
/**
 * @name contributions
 * @description Retrieves all contributions for a given parachain crowdloan.
 * @param {string | number | BN} paraId The parachain ID for which contributions are being queried.
 * @example
 * ```javascript
 * const contributions = await api.derive.crowdloan.contributions(3369);
 * console.log("Contributions:", contributions);
 * ```
 */
export declare function contributions(instanceId: string, api: DeriveApi): (paraId: string | number | BN) => Observable<DeriveContributions>;
