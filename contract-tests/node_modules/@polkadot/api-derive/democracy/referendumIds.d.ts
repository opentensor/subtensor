import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi } from '../types.js';
/**
 * @name referendumIds
 * @description Retrieves an array of active referendum IDs.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumIds();
 * ```
 */
export declare function referendumIds(instanceId: string, api: DeriveApi): () => Observable<BN[]>;
