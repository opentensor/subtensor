import { Float } from '../native/Float.js';
declare const f64_base: import("../types/codec.js").CodecClass<Float, any[]>;
/**
 * @name f64
 * @description
 * A 64-bit float
 */
export declare class f64 extends f64_base {
    readonly __FloatType = "f64";
}
export {};
