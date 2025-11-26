import { UInt } from '../base/UInt.js';
/**
 * @name u32
 * @description
 * A 32-bit unsigned integer
 */
export class u32 extends UInt.with(32) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u32';
}
