import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveCouncilVotes } from '../types.js';
/**
 * @name votes
 * @description Retrieves the council election votes for all participants.
 * @example
 * ```javascript
 * const votes = await api.derive.council.votes();
 * ```
 */
export declare function votes(instanceId: string, api: DeriveApi): () => Observable<DeriveCouncilVotes>;
