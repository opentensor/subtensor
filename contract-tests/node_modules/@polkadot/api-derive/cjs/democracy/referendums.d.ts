import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveReferendumExt } from '../types.js';
/**
 * @name referendums
 * @description Retrieves information about all active referendums, including their details and associated votes.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendums();
 * ```
 */
export declare function referendums(instanceId: string, api: DeriveApi): () => Observable<DeriveReferendumExt[]>;
