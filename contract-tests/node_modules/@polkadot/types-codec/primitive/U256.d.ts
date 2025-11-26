import { UInt } from '../base/UInt.js';
declare const u256_base: import("../types/codec.js").CodecClass<UInt, any[]>;
/**
 * @name u256
 * @description
 * A 256-bit unsigned integer
 */
export declare class u256 extends u256_base {
    readonly __UIntType = "u256";
}
export {};
