import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveReferendum } from '../types.js';
/**
 * @name referendumsActive
 * @description Retrieves information about active referendums.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumsActive();
 * console.log("Active Referendums:", referendums);
 * ```
 */
export declare function referendumsActive(instanceId: string, api: DeriveApi): () => Observable<DeriveReferendum[]>;
