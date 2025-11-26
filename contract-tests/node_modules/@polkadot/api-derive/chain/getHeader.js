import { map, switchMap } from 'rxjs';
import { createHeaderExtended } from '../type/index.js';
import { memo } from '../util/index.js';
import { getAuthorDetails } from './util.js';
/**
 * @name getHeader
 * @param {( Uint8Array | string )} hash - A block hash as U8 array or string.
 * @returns An array containing the block header and the block author
 * @description Get a specific block header and extend it with the author
 * @example
 * ```javascript
 * const { author, number } = await api.derive.chain.getHeader('0x123...456');
 *
 * console.log(`block #${number} was authored by ${author}`);
 * ```
 */
export function getHeader(instanceId, api) {
    return memo(instanceId, (blockHash) => api.rpc.chain.getHeader(blockHash).pipe(switchMap((header) => getAuthorDetails(api, header, blockHash)), map(([header, validators, author]) => createHeaderExtended((validators || header).registry, header, validators, author))));
}
