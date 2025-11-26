import type { Observable } from 'rxjs';
import type { BN } from '@polkadot/util';
import type { DeriveApi } from '../types.js';
/**
 * @name sqrtElectorate
 * @description Computes the square root of the total token issuance in the network.
 * @example
 * ```javascript
 * let sqrtElectorate = await api.derive.democracy.sqrtElectorate();
 * console.log("Square root of token issuance:", sqrtElectorate);
 * ```
 */
export declare function sqrtElectorate(instanceId: string, api: DeriveApi): () => Observable<BN>;
