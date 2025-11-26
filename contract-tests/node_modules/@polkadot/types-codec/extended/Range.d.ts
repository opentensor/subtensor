import type { AnyTuple, CodecClass, INumber, Registry } from '../types/index.js';
import { Tuple } from '../base/Tuple.js';
type RangeType = 'Range' | 'RangeInclusive';
interface Options {
    rangeName?: RangeType;
}
/**
 * @name Range
 * @description
 * Rust `Range<T>` representation
 */
export declare class Range<T extends INumber> extends Tuple {
    #private;
    constructor(registry: Registry, Type: CodecClass<T> | string, value?: AnyTuple, { rangeName }?: Options);
    static with<T extends INumber>(Type: CodecClass<T> | string): CodecClass<Range<T>>;
    /**
     * @description Returns the starting range value
     */
    get start(): T;
    /**
     * @description Returns the ending range value
     */
    get end(): T;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
}
export {};
