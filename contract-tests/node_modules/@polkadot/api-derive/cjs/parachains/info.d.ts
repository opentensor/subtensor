import type { Observable } from 'rxjs';
import type { ParaId } from '@polkadot/types/interfaces';
import type { DeriveApi, DeriveParachainFull } from '../types.js';
/**
 * @name info
 * @param {ParaId | number} id A unique numeric (non-negative integer) identifier for a parachain.
 * @description Retrieves detailed information about a specific parachain.
 * @example
 * ```javascript
 * await api.derive.parachains.info(1000, (info) => {
 *   if (info) {
 *     console.log(`Parachain ${info.id.toString()} is active: ${info.active}`);
 *   } else {
 *     console.log("Parachain information not available.");
 *   }
 * });
 * ```
*/
export declare function info(instanceId: string, api: DeriveApi): (id: ParaId | number) => Observable<DeriveParachainFull | null>;
