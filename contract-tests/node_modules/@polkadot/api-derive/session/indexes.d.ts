import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveSessionIndexes } from '../types.js';
/**
 * @name indexes
 * @description Retrieves session-related index data, adapting to whether
 * the chain has staking enabled.
 * @example
 * ```javascript
 * api.derive.session.indexes((indexes) => {
 *   console.log(`Current session index: ${indexes.currentIndex}`);
 *   console.log(`Validator count: ${indexes.validatorCount}`);
 * });
 * ```
 */
export declare function indexes(instanceId: string, api: DeriveApi): () => Observable<DeriveSessionIndexes>;
