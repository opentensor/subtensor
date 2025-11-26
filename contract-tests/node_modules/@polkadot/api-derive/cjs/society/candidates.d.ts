import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveSocietyCandidate } from '../types.js';
/**
 * @name candidate
 * @description Retrieves the list of candidates for the society module.
 * @example
 * ```javascript
 * const societyCandidates = await api.derive.society.candidates();
 * console.log(societyCandidates);
 * ```
 */
export declare function candidates(instanceId: string, api: DeriveApi): () => Observable<DeriveSocietyCandidate[]>;
