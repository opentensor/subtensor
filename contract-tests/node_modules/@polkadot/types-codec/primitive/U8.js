import { UInt } from '../base/UInt.js';
/**
 * @name u8
 * @description
 * An 8-bit unsigned integer
 */
export class u8 extends UInt.with(8) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u8';
}
