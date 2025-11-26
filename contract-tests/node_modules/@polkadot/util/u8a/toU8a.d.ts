import type { U8aLike } from '../types.js';
/**
 * @name u8aToU8a
 * @summary Creates a Uint8Array value from a Uint8Array, Buffer, string or hex input.
 * @description
 * `null` or `undefined` inputs returns a `[]` result, Uint8Array values returns the value, hex strings returns a Uint8Array representation.
 * If `strict` is true, `null` or `undefined` will throw an error instead of returning an empty array.
 * Supports input types: Uint8Array, Buffer, hex string, string, or number array.
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aToU8a } from '@polkadot/util';
 *
 * u8aToU8a(new Uint8Array([0x12, 0x34]); // => Uint8Array([0x12, 0x34])
 * u8aToU8a(0x1234); // => Uint8Array([0x12, 0x34])
 * ```
 */
export declare function u8aToU8a(value?: U8aLike | null, strict?: boolean): Uint8Array;
