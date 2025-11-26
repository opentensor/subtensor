import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveBounties } from '../types.js';
/**
 * @name bounties
 * @descrive Retrieves all active bounties, their descriptions, and associated proposals.
 * @example
 * ```javascript
 * const bounties = await api.derive.bounties();
 * console.log("Active bounties:", bounties);
 * ```
 */
export declare function bounties(instanceId: string, api: DeriveApi): () => Observable<DeriveBounties>;
