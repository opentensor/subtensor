import type { BufferObject } from '../types.js';
/**
 * @name u8aToBuffer
 * @summary Creates a Buffer object from a hex string.
 * @description
 * `null` inputs returns an empty `Buffer` result. `UInt8Array` input values return the actual bytes value converted to a `Buffer`. Anything that is not a `UInt8Array` throws an error.
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aToBuffer } from '@polkadot/util';
 *
 * console.log('Buffer', u8aToBuffer(new Uint8Array([1, 2, 3])));
 * ```
 */
export declare function u8aToBuffer<T = BufferObject>(value?: Uint8Array | null): T;
