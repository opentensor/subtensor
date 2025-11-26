import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveSociety } from '../types.js';
/**
 * @name info
 * @description Get the overall info for a society.
 * @example
 * ```javascript
 * const societyInfo = await api.derive.society.candidates();
 * console.log(societyInfo);
 * ```
 */
export declare function info(instanceId: string, api: DeriveApi): () => Observable<DeriveSociety>;
