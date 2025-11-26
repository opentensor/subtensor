import type { U8aLike } from '../types.js';
import { BN } from '../bn/index.js';
/**
 * @name compactFromU8a
 * @description Retrives the offset and encoded length from a compact-prefixed value
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactFromU8a } from '@polkadot/util';
 *
 * const [offset, length] = compactFromU8a(new Uint8Array([254, 255, 3, 0]));
 *
 * console.log('value offset=', offset, 'length=', length); // 4, 0xffff
 * ```
 */
export declare function compactFromU8a(input: U8aLike): [number, BN];
/**
 * @name compactFromU8aLim
 * @description A limited version of [[compactFromU8a]], accepting only Uint8Array inputs for values <= 48 bits
 */
export declare function compactFromU8aLim(u8a: Uint8Array): [number, number];
