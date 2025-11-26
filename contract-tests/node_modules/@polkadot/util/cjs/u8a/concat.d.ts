import type { U8aLike } from '../types.js';
/**
 * @name u8aConcat
 * @summary Creates a concatenated Uint8Array from the inputs.
 * @description
 * Concatenates the input arrays into a single `UInt8Array`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { { u8aConcat } from '@polkadot/util';
 *
 * u8aConcat(
 *   new Uint8Array([1, 2, 3]),
 *   new Uint8Array([4, 5, 6])
 * ); // [1, 2, 3, 4, 5, 6]
 * ```
 */
export declare function u8aConcat(...list: readonly U8aLike[]): Uint8Array;
/**
 * @name u8aConcatStrict
 * @description A strict version of [[u8aConcat]], accepting only Uint8Array inputs
 */
export declare function u8aConcatStrict(u8as: readonly Uint8Array[], length?: number): Uint8Array;
