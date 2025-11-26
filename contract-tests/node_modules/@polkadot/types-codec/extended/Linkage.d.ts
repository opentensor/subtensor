import type { Codec, CodecClass, Registry } from '../types/index.js';
import { Option } from '../base/Option.js';
import { Tuple } from '../base/Tuple.js';
import { Struct } from '../native/Struct.js';
type TypeWithValues = [CodecClass, any[]];
/**
 * @name Linkage
 * @description The wrapper for the result from a LinkedMap
 */
export declare class Linkage<T extends Codec> extends Struct {
    constructor(registry: Registry, Type: CodecClass | string, value?: unknown);
    static withKey<O extends Codec>(Type: CodecClass | string): CodecClass<Linkage<O>>;
    /**
     * @description Returns the next item the Linkage is pointing to
     */
    get previous(): Option<T>;
    /**
     * @description Returns the previous item the Linkage is pointing to
     */
    get next(): Option<T>;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Custom toU8a which with bare mode does not return the linkage if empty
     */
    toU8a(isBare?: boolean): Uint8Array;
}
/**
 * @name LinkageResult
 * @description A Linkage keys/Values tuple
 */
export declare class LinkageResult extends Tuple {
    constructor(registry: Registry, [TypeKey, keys]: TypeWithValues, [TypeValue, values]: TypeWithValues);
}
export {};
