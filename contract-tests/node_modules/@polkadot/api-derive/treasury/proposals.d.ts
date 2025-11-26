import type { Observable } from 'rxjs';
import type { DeriveApi, DeriveTreasuryProposals } from '../types.js';
/**
 * @name proposals
 * @description Retrieve all active and approved treasury proposals, along with their info.
 * @example
 * ```javascript
 * const treasuryProposals = await api.derive.treasury.proposals();
 * console.log(treasuryProposals);
 * ```
 */
export declare function proposals(instanceId: string, api: DeriveApi): () => Observable<DeriveTreasuryProposals>;
