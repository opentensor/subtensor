import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveDispatch } from '../types.js';
/**
 * @name dispatchQueue
 * @description Retrieves the list of scheduled or pending dispatches in the governance system.
 * @example
 * ```javascript
 * const queue = await api.derive.democracy.dispatchQueue();
 * console.log("Dispatch Queue:", queue);
 * ```
 */
export declare function dispatchQueue(instanceId: string, api: DeriveApi): () => Observable<DeriveDispatch[]>;
