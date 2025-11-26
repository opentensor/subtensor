import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveProposalExternal } from '../types.js';
/**
 * @name nextExternal
 * @description Retrieves the next external proposal that is scheduled for a referendum.
 * @example
 * ```javascript
 * const nextExternal = await api.derive.democracy.nextExternal();
 * console.log("Next external proposal:", nextExternal);
 * ```
 */
export declare function nextExternal(instanceId: string, api: DeriveApi): () => Observable<DeriveProposalExternal | null>;
