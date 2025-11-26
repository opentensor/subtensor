import type { Codec, CodecClass, Registry } from '../types/index.js';
/**
 * @internal
 * From a type string or class, return the associated type class
 */
export declare function typeToConstructor<T extends Codec = Codec>(registry: Registry, type: string | CodecClass<T>): CodecClass<T>;
/**
 * @internal
 * Takes an input array of types and returns the associated classes for it
*/
export declare function typesToConstructors<T extends Codec = Codec>(registry: Registry, types: (string | CodecClass<T>)[]): CodecClass<T>[];
/**
 * @internal
 * Takes an input map of the form `{ [string]: string | CodecClass }` and returns a map of `{ [string]: CodecClass }`
 */
export declare function mapToTypeMap(registry: Registry, input: Record<string, string | CodecClass>): [CodecClass[], string[]];
