import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveHeartbeats } from '../types.js';
/**
 * @name receivedHeartbeats
 * @description Return a boolean array indicating whether the passed accounts had received heartbeats in the current session.
 * @example
 * ```javascript
 * let unsub = await api.derive.imOnline.receivedHeartbeats((heartbeat) => {
 *   console.log(heartbeat);
 * });
 * ```
 */
export declare function receivedHeartbeats(instanceId: string, api: DeriveApi): () => Observable<DeriveHeartbeats>;
