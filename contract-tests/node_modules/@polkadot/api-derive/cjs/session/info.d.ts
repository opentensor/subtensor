import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveSessionInfo } from '../types.js';
/**
 * @name info
 * @description Retrieves all the session and era query and calculates specific values on it as the length of the session and eras.
 * @example
 * ```javascript
 * api.derive.session.info((info) => {
 *   console.log(`Session info ${JSON.stringify(info)}`);
 * });
 * ```
 */
export declare function info(instanceId: string, api: DeriveApi): () => Observable<DeriveSessionInfo>;
