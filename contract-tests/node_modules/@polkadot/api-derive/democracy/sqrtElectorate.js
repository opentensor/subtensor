import { map } from 'rxjs';
import { bnSqrt } from '@polkadot/util';
import { memo } from '../util/index.js';
/**
 * @name sqrtElectorate
 * @description Computes the square root of the total token issuance in the network.
 * @example
 * ```javascript
 * let sqrtElectorate = await api.derive.democracy.sqrtElectorate();
 * console.log("Square root of token issuance:", sqrtElectorate);
 * ```
 */
export function sqrtElectorate(instanceId, api) {
    return memo(instanceId, () => api.query.balances.totalIssuance().pipe(map(bnSqrt)));
}
