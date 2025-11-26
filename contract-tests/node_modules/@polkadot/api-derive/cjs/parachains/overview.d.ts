import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveParachain } from '../types.js';
/**
 * @name overview
 * @description Retrieves an overview of all registered parachains.
 * @example
 * ```javascript
 * await api.derive.parachains.overview((overview) => {
 *   parachains.forEach(parachain => {
 *     console.log(`Parachain ${parachain.id.toString()} is registered.`);
 *   });
 * });
 * ```
 */
export declare function overview(instanceId: string, api: DeriveApi): () => Observable<DeriveParachain[]>;
