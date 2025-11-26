import type { Codec } from '../types/index.js';
import { AbstractArray } from '../abstract/Array.js';
/**
 * @name VecAny
 * @description
 * This manages codec arrays, assuming that the inputs are already of type Codec. Unlike
 * a vector, this can be used to manage array-like structures with variable arguments of
 * any types
 */
export declare class VecAny<T extends Codec> extends AbstractArray<T> {
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
}
