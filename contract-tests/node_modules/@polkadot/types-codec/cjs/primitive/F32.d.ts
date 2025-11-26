import { Float } from '../native/Float.js';
declare const f32_base: import("../types/codec.js").CodecClass<Float, any[]>;
/**
 * @name f32
 * @description
 * A 32-bit float
 */
export declare class f32 extends f32_base {
    readonly __FloatType = "f32";
}
export {};
