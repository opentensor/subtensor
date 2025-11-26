import type { Codec, CodecClass, Registry } from '../types/index.js';
/**
 * @internal
 *
 * Given an u8a, and an array of Type constructors, decode the u8a against the
 * types, and return an array of decoded values.
 *
 * @param u8a - The u8a to decode.
 * @param result - The result array (will be returned with values pushed)
 * @param types - The array of CodecClass to decode the U8a against.
 */
export declare function decodeU8a<T extends Codec = Codec>(registry: Registry, result: T[], u8a: Uint8Array, [Types, keys]: [CodecClass<T>[], string[]]): [T[], number];
/**
 * @internal
 *
 * Split from decodeU8a since this is specialized to zip returns ... while we duplicate, this
 * is all on the hot-path, so it is not great, however there is (some) method behind the madness
 */
export declare function decodeU8aStruct(registry: Registry, result: [string, Codec][], u8a: Uint8Array, [Types, keys]: [CodecClass[], string[]]): [[string, Codec][], number];
/**
 * @internal
 *
 * Split from decodeU8a since this is specialized to 1 instance ... while we duplicate, this
 * is all on the hot-path, so it is not great, however there is (some) method behind the madness
 */
export declare function decodeU8aVec<T extends Codec = Codec>(registry: Registry, result: unknown[], u8a: Uint8Array, startAt: number, Type: CodecClass<T>): [number, number];
