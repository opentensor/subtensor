import type { Observable } from 'rxjs';
import type { PalletDemocracyReferendumInfo } from '@polkadot/types/lookup';
import type { DeriveApi } from '../types.js';
type ReferendumInfoFinished = PalletDemocracyReferendumInfo['asFinished'];
/**
 * @name referendumsFinished
 * @description Retrieves information about finished referendums.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumsFinished();
 * console.log("Finished Referendums:", referendums);
 * ```
 */
export declare function referendumsFinished(instanceId: string, api: DeriveApi): () => Observable<ReferendumInfoFinished[]>;
export {};
