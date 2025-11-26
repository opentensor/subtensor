import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveProposal } from '../types.js';
/**
 * @name proposals
 * @description Retrieves the list of active public proposals in the democracy module, along with their associated preimage data and deposit information.
 * @example
 * ```javascript
 * const proposals = await api.derive.democracy.proposals();
 * console.log("proposals:", proposals);
 * ```
 */
export declare function proposals(instanceId: string, api: DeriveApi): () => Observable<DeriveProposal[]>;
