import { map } from 'rxjs';
import { objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
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
export function info(instanceId, api) {
    return memo(instanceId, () => api.derive.session.indexes().pipe(map((indexes) => {
        const sessionLength = api.consts?.babe?.epochDuration || api.registry.createType('u64', 1);
        const sessionsPerEra = api.consts?.staking?.sessionsPerEra || api.registry.createType('SessionIndex', 1);
        return objectSpread({
            eraLength: api.registry.createType('BlockNumber', sessionsPerEra.mul(sessionLength)),
            isEpoch: !!api.query.babe,
            sessionLength,
            sessionsPerEra
        }, indexes);
    })));
}
