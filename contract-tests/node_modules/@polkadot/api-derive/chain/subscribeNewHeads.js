import { map, switchMap } from 'rxjs';
import { createHeaderExtended } from '../type/index.js';
import { memo } from '../util/index.js';
import { getAuthorDetails } from './util.js';
/**
 * @name subscribeNewHeads
 * @returns A header with the current header (including extracted author).
 * @description An observable of the current block header and it's author.
 * @example
 * ```javascript
 * api.derive.chain.subscribeNewHeads((header) => {
 *   console.log(`block #${header.number} was authored by ${header.author}`);
 * });
 * ```
 */
export function subscribeNewHeads(instanceId, api) {
    return memo(instanceId, () => api.rpc.chain.subscribeNewHeads().pipe(switchMap((header) => getAuthorDetails(api, header)), map(([header, validators, author]) => {
        header.createdAtHash = header.hash;
        return createHeaderExtended(header.registry, header, validators, author);
    })));
}
